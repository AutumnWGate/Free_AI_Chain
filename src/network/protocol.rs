use crate::network::error::Error;
use libp2p::{
    core::muxing::StreamMuxerBox,
    core::transport::Boxed,
    futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, StreamExt},
    request_response::{self, Behaviour, Codec, Config, Event, ProtocolSupport},
    swarm::SwarmEvent,
    PeerId, SwarmBuilder,

};

use std::{iter, time::Duration};
use crate::types::{message::Request, message::Response};
use crate::crypto::hash::Hash;


#[derive(Debug, Clone)]
pub struct FaicProtocol();

impl FaicProtocol {
    ///返回协议名称的字节切片
    fn protocol_name(&self) -> &[u8] {
        "/faic/1".as_bytes()
    }
}

impl AsRef<str> for FaicProtocol {
    ///返回协议名称的字符串引用
    fn as_ref(&self) -> &str {
        "/faic/1"
    }
}

#[derive(Clone)]
pub struct FaicCodec();

// 为 FaicCodec 结构体实现异步 Codec trait。
#[async_trait::async_trait]
impl Codec for FaicCodec {
    type Protocol = FaicProtocol;
    // 定义关联类型 Request 为 crate::types::message::Request。
    type Request = Request;
    // 定义关联类型 Response 为 crate::types::message::Response。
    type Response = Response;

    // 实现异步 read_request 方法，用于从流中读取请求。
    async fn read_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> std::io::Result<Self::Request>
    // 注意返回类型改为 std::io::Result
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        serde_json::from_slice(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    // 实现异步 write_request 方法，用于将请求写入流中。
    async fn write_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> std::io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        // 将请求序列化为 JSON 字节切片
        let buf = serde_json::to_vec(&req)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        // 将字节切片写入流
        io.write_all(&buf).await?;
        // 刷新流，确保所有数据都已发送
        io.flush().await
    }

    // 实现异步 read_response 方法，用于从流中读取响应。
    async fn read_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> std::io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        // 创建一个空的字节向量来存储响应数据
        let mut buf = Vec::new();
        // 从流中读取数据到 buf
        io.read_to_end(&mut buf).await?;
        // 将 buf 反序列化为 Response 类型
        serde_json::from_slice(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    // 实现异步 write_response 方法，用于将响应写入流中。
    async fn write_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> std::io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        // 将响应序列化为 JSON 字节切片
        let buf = serde_json::to_vec(&res)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        // 将字节切片写入流
        io.write_all(&buf).await?;
        // 刷新流，确保所有数据都已发送
        io.flush().await
    }
}

// 创建 FaicNetworkBehaviour，返回一个 Behaviour<FaicCodec>
pub fn create_faic_network_behaviour() -> Behaviour<FaicCodec> {
    let protocols = iter::once((
        FaicProtocol(), // 使用 FaicProtocol()
        ProtocolSupport::Full,
    ));

    // 创建 Config 对象，设置请求超时时间为 10 秒
    let config = Config::default().with_request_timeout(Duration::from_secs(10));

    // 使用 FaicCodec 和 protocols 创建 Behaviour
    Behaviour::with_codec(FaicCodec(), protocols, config)
}

// 处理请求，返回一个 Result<Response, Error>
pub async fn handle_request(request: Request) -> Result<Response, Error> {
    // 根据请求类型匹配处理逻辑
    match request {
        // 处理获取余额请求
        Request::GetBalance { address } => {
            // 调用 ledger 模块的 get_balance 函数获取余额
            let balance = crate::ledger::get_balance(&address).await?;
            // 返回包含余额的响应
            Ok(Response::GetBalanceResponse { balance })
        }
        // 处理发送交易请求
        Request::SendTransaction { transaction } => {
            // 调用 ledger 模块的 validate_transaction 函数验证交易
            crate::ledger::validate_transaction(&transaction).await?;
            // 调用 ledger 模块的 add_transaction_to_pool 函数将交易添加到交易池
            crate::ledger::add_transaction_to_pool(&transaction).await?;
            // 返回包含交易哈希的响应
            Ok(Response::SendTransactionResponse {
                transaction_hash: transaction.transaction_hash,
            })
        }
        // 处理获取节点信息请求
        Request::GetNodeInfo => {
            // 调用 network 模块的 get_node_info 函数获取节点信息
            let node_info = crate::network::get_node_info().await?;
            // 返回包含节点信息的响应
            Ok(Response::GetNodeInfoResponse(node_info))
        }
        // 处理获取 Merkle 证明请求
        Request::GetMerkleProof {
            block_hash,
            transaction_hash,
        } => {
            // 将 block_hash 转换为十六进制字符串。
            let block_hash_str = hex::encode(&block_hash);
            // 调用 crate::ledger::get_block 获取区块。
            let block = crate::ledger::get_block(&block_hash_str).await?;
            // 查找指定的交易。
            let transaction = block
                .transactions
                .iter()
                .find(|transaction| transaction.hash == transaction_hash)
                .ok_or(Error::NotFound)?;
            // 将 Merkle 证明转换为 Vec<Vec<u8>>。
            let merkle_proof: Vec<Hash> = block
                .merkle_proof
                .iter()
                .map(|hash| *hash) 
                .collect();
            Ok(Response::GetMerkleProofResponse { merkle_proof })
        }
        // 处理 GetBlock 请求。
        Request::GetBlock { block_hash } => {
            // block_hash 现在是 Hash 类型，直接使用
            let block = crate::ledger::get_block(block_hash).await?;
            // 返回 BlockResponse 响应。
            Ok(Response::GetBlockResponse { block })
        }
        // 处理 GetBlockByHeight 请求。
        Request::GetBlockByHeight { height } => {
            // 调用 crate::ledger::get_block_by_height 根据高度获取区块。
            let block = crate::ledger::get_block_by_height(height).await?;
            // 返回 BlockResponse 响应。
            Ok(Response::GetBlockByHeightResponse { block })
        }
        // 处理 GetLatestBlock 请求。
        Request::GetLatestBlock => {
            // 调用 crate::ledger::get_latest_block 获取最新区块。
            let block = crate::ledger::get_latest_block().await?;
            // 返回 LatestBlockResponse 响应。
            Ok(Response::GetLatestBlockResponse { block })
        }
    }
}

// 定义一个名为 start_listening 的异步函数，启动监听并返回 Result<(), Error>。
pub async fn start_listening(_transport: Boxed<(PeerId, StreamMuxerBox)>) -> Result<(), Error> {
    // 创建 NetworkBehaviour
    let behaviour = create_faic_network_behaviour();

    // 直接生成 libp2p Secp256k1 密钥对
    let keypair = libp2p::identity::Keypair::generate_secp256k1();
    let peer_id = keypair.public().to_peer_id();
    
    println!("Local peer id: {}", peer_id);


    // 构建 Swarm
    let mut swarm = SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            libp2p::tcp::Config::default(),
            libp2p::noise::Config::new,
            libp2p::yamux::Config::default,
        )?  // 直接使用 ? 操作符
        .with_behaviour(|_| behaviour)
        .expect("Behaviour creation cannot fail") // 使用 expect，因为这里不应该失败
        .build();

    // 监听所有接口
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().map_err(Error::from)?)?;

    // 启动一个异步任务来处理事件
    tokio::spawn(async move {
        loop {
            tokio::select! {
                event = swarm.select_next_some() => match event {
                    // 处理来自对等节点的消息事件。
                    SwarmEvent::Behaviour(Event::Message { peer, message, connection_id:_ }) => {
                        match message {
                            request_response::Message::Request {
                                request,
                                channel,
                                ..
                            } => {
                                println!("Received request from {:?}: {:?}", peer, request);
                                // 调用 handle_request 处理请求。
                                let response = match handle_request(request).await {
                                    Ok(response) => response,
                                    Err(e) => Response::Error {
                                        message: e.to_string()
                                    }
                                };
                                // 将响应发送回请求的节点。
                                if let Err(e) = swarm.behaviour_mut().send_response(channel, response) {
                                    eprintln!("Failed to send response: {:?}", e);
                                }
                            }
                            // 处理响应消息。
                            request_response::Message::Response {
                                response,
                                request_id,
                                ..
                            } => {
                                println!("Received response from {:?}: {:?} (request_id: {:?})", peer, response, request_id);
                            }
                        }
                    }
                    // 处理出站失败事件。
                    SwarmEvent::Behaviour(Event::OutboundFailure { peer, error, .. }) => {
                        eprintln!("Outbound failure to {:?}: {:?}", peer, error);
                    }
                    // 处理入站失败事件。
                    SwarmEvent::Behaviour(Event::InboundFailure { peer, error, .. }) => {
                        eprintln!("Inbound failure from {:?}: {:?}", peer, error);
                    }
                    // 处理响应发送事件。
                    SwarmEvent::Behaviour(Event::ResponseSent { peer, .. }) => {
                        println!("Response sent to {:?}", peer);
                    }
                    _ => {}
                }
            }
        }
    });
    Ok(())
}
