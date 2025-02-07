# FAIC_Protocol 通讯协议

## 1. 概述

本文档描述了 FAIC 网络中节点间通信的协议，包括消息格式、编解码方式、请求和响应的处理逻辑等。该协议是 `discovery`、`ledger`、`connection` 和 `handler` 模块的基础。

## 2. 依赖模块

*   **`types`**: 定义了基本的数据类型，如 `node`、`message`、`amount` 、`transaction`、``。
*   **`error`**: 定义了错误类型。
*   **`config`**: 定义了网络配置。
*   **`ledger`**: **定义了账本相关的数据结构和接口，包括 `TransactionDetail`、`AccountState`、`Block`、`Storage` 等。**


## 3. 消息定义
### 3.1. MessageType
消息类型 用于区分不同类型的消息，包含以下几种：
*   `Request`: 表示一个节点向另一个节点发起请求。
*   `Response`: 表示一个节点对另一个节点请求的回复。
*   `Heartbeat`: 用于检测节点是否在线。
*   `Data`: 用于传输数据。

### 3.2. Request
请求类型包括：
*   `GetBalance`: 查询余额，参数为钱包地址。
    *   `address`: 要查询的钱包地址，String 类型，需要符合 BIP32、BIP39、BIP44、BIP47 和 BIP84 标准。
*   `SendTransaction`: 发送交易，参数为交易内容。
    *   `transaction`: 要发送的交易，类型为 `TransactionDetail`。
*   `GetNodeInfo`: 获取节点信息。
*   `GetMerkleProof`: 获取 Merkle Proof，参数为区块哈希和交易哈希。
    *   `block_hash`: 要获取的区块哈希，String 类型。
    *   `transaction_hash`: 要获取的交易哈希，String 类型。  

### 3.3. Response
响应类型 用于表示不同类型的响应，包含以下几种：
*   `GetBalanceResponse`: 对 GetBalance 请求的响应。包含一个参数：
    *   `available_balance`: 可用余额，Amount 类型。
    *   `locked_balance`: 锁定余额，Amount 类型。
*   `SendTransactionResponse`: 对 SendTransaction 请求的响应。包含一个参数：
    *   `transaction_hash`: 交易哈希，[u8; 32] 类型。
*   `GetNodeInfoResponse`: 对 GetNodeInfo 请求的响应。包含一个参数：

    *   `node_info`: 节点信息，NodeInfo 类型。
*   `Error`: 表示请求处理过程中发生了错误。包含一个参数：
    *   `message`: 错误信息，String 类型。
*   `GetMerkleProofResponse`: 对 GetMerkleProof 请求的响应。包含一个参数：
    *   `merkle_proof`: Merkle Proof，Vec<Vec<u8>> 类型。

### 3.4. Transaction
交易结构体 用于表示一笔交易，包含以下字段：
*   `from`: 交易发送方地址，String 类型，符合 BIP32、BIP39、BIP44、BIP47 和 BIP84 标准。
*   `to`: 交易接收方地址，String 类型，符合 BIP32、BIP39、BIP44、BIP47 和 BIP84 标准。
*   `transfer_amount`: 交易金额，Amount 类型。
*   `fee`: 交易费用，Amount 类型。
*   `timestamp`: 交易时间戳 (Unix 时间戳, 单位为秒)，u64 类型。
*   `signature`: 交易发送方签名，Vec<u8> 类型，使用发送方地址对应的私钥对交易信息进行签名，签名算法采用 ECDSA，曲线为 secp256k1。
*   `hash`: 交易哈希，[u8; 32] 类型，使用 SHA256 算法对交易信息进行哈希计算。
*   `transaction_type`: 交易类型，具体类型定义见 ledger 模块的 TransactionType 枚举。
*   `nonce`: 交易序号，u64 类型，用于防止重放攻击。

### 3.5. NodeInfo
节点信息 用于表示一个节点的信息，包含以下字段：
*   `peer_id`: 节点的 PeerId，使用 crate::network::config::serde_peer_id 进行序列化和反序列化。
*   `address`: 节点的地址列表，Vec<Multiaddr> 类型。
*   `is_online`: 节点是否在线，bool 类型。
*   `peer_version`: 节点版本，String 类型。
*   `node_manager_wallet_address`: 节点管理者钱包地址，String 类型。


### 3.6. Amount 
构建在/src/types/mod.rs 中。
Amount 数据类型 用于表示 FAIC 代币的数量，基础数据类型为 BigUint，精度为小数点后 8 位。
*   1 FAIC = 10^8 个最小单位。
*   最大数量: 2^128 - 1。
*   最小单位: 1 (0.00000001 FAIC), 实际精度: 8位小数。参考来源doge
*   `DECIMALS`: 常量，值为 8，表示小数点后的位数。
*   `ONE_FAIC`: 常量，值为 BigUint::from(100_000_000u64)，表示 1 FAIC。
*   `MAX_AMOUNT`: 常量，值为 BigUint::parse_bytes(b"340282366920938463463374607431768211455", 10)，表示最大数量。
*   `from_biguint` 方法: 从 BigUint 创建 Amount，如果值大于 MAX_AMOUNT，则返回错误。
*   `from_str` 方法: 从字符串创建 Amount，如果字符串无法解析为 BigUint，则返回错误。
*   `value` 方法: 获取 Amount 的值，返回 &BigUint。
*   `to_string` 方法: 将 Amount 转换为字符串，包含八位小数。例如：
    *   如果值为 12345678，则返回 "0.12345678"。
    *   如果值为 1234567890，则返回 "12.34567890"。

### 3.7. Error
构建在/src/network/error.rs 中。
错误类型 用于表示各种错误情况，包含以下几种：
*   `InvalidAddress`: 无效的地址。
*   `InsufficientBalance`: 余额不足。
*   `InvalidSignature`: 无效的签名。
*   `InvalidTimestamp`: 无效的时间戳。
*   `InvalidHash`: 无效的哈希。
*   `InvalidNonce`: 无效的交易序号。
*   `AddTransactionToPoolFailed`: 交易添加到交易池失败。
*   `GetNodeInfoFailed`: 获取节点信息失败。
*   `SerializationError`: 序列化错误，包含具体的错误信息。
*   `DeserializationError`: 反序列化错误，包含具体的错误信息。
*   `DatabaseError`: 数据库错误，包含具体的错误信息。
*   `Other`: 其他错误，包含具体的错误信息。
*   `NotFound`: 找不到数据。
*   `InvalidMerkleProof`: 无效的 Merkle Proof。
*   `MerkleTreeError`: 构建 Merkle Tree 错误。
   
### 4. 编解码
使用 serde_json 进行序列化和反序列化。
FaicCodec 实现了 RequestResponseCodec trait，用于编解码请求和响应。
*   `read_request` 方法: 从输入流中读取请求数据，反序列化为 Request 类型。
*   `read_response` 方法: 从输入流中读取响应数据，反序列化为 Response 类型。
*   `write_request` 方法: 将 Request 类型序列化为字节流，写入输出流。
*   `write_response` 方法: 将 Response 类型序列化为字节流，写入输出流。

### 5. 处理请求
protocol 模块需要实现 handle_request 函数来处理请求。
handle_request 函数根据不同的请求类型进行处理，并返回相应的响应。

#### 5.1. GetBalance
*   请求参数: `address` (String)
*   响应: `GetBalanceResponse` 或 `Response::Error`
*   处理逻辑:
    1.  调用 ledger 模块的 `get_balance` 函数查询 `address` 对应的余额。
    2.  如果查询成功，返回 `GetBalanceResponse`，包含查询到的余额。
    3.  如果查询失败，返回 `Response::Error`，message 字段包含具体的错误信息，例如 "Account not found" 等。

#### 5.2. SendTransaction
*   请求参数: `transaction` (Transaction)
*   响应: `SendTransactionResponse` 或 `Response::Error`
*   处理逻辑:
    1.  调用账本模块的 `validate_transaction` 函数验证交易的有效性：
        *   验证 `from` 地址的 `balance` 余额是否足够支付 `transfer_amount + fee`。
        *   验证 `signature` 是否有效。
        *   验证 `timestamp` 是否在合理范围内。
        *   验证 `hash` 是否正确。
        *   验证 `nonce` 是否与 `from` 地址的当前 `nonce` 值一致。
    2.  调用账本模块的 `add_transaction_to_pool` 函数将交易添加到交易池。
    3.  如果验证和添加都成功，返回 `SendTransactionResponse`，包含交易哈希。
    4.  如果验证失败或添加失败，返回 `Response::Error`，message 字段包含具体的错误信息，例如 "Insufficient balance"、"Invalid signature" 等。

#### 5.3. GetNodeInfo
*   请求参数: 无
*   响应: `GetNodeInfoResponse` 或 `Response::Error`
*   处理逻辑:
        1. 调用 `network` 模块的 `get_node_info` 函数获取本地节点的 `NodeInfo`。
        2. 返回 `GetNodeInfoResponse`，包含 `NodeInfo`。
        3. 如果获取节点信息失败，返回 `Response::Error`，message 字段包含具体的错误信息。

#### 5.4. GetMerkleProof
*   请求参数: `block_hash` (String), `transaction_hash` (String)
*   响应: `GetMerkleProofResponse` 或 `Response::Error`
*   处理逻辑:
        1. 从账本模块获取指定 `block_hash` 的区块。
        2. 在区块中查找 `transaction_hash` 对应的交易。
        3. 生成该交易的 Merkle Proof。
        4. 返回 `GetMerkleProofResponse`，包含 Merkle Proof。
        5. 如果在获取区块、查找交易或生成 Merkle Proof 过程中出现错误，返回 `Response::Error`，message 字段包含具体的错误信息。

#### 5.5. TransactionType:
TransactionType 枚举定义了交易类型，包含以下几种：
*   `Transfer`: 普通交易。
*   `SmartContract`: 智能合约。

### 6. 与其他模块的集成
#### 6.1. `discovery`模块可以使用 `protocol` 模块来广播 `GetNodeInfo` 请求，以发现网络中的其他节点。
#### 6.2. `connection`模块可以使用 `protocol` 模块来建立与其他节点的连接，并发送和接收请求和响应。
#### 6.3. `handler`模块负责调用 `protocol` 模块的 `handle_request` 函数来处理接收到的请求。
#### 6.4. `ledger`模块负责维护账本状态，包括账户余额、交易记录等。`protocol` 模块通过调用 `ledger` 模块提供的接口来查询余额、验证交易和添加交易到交易池。

### 7. 转账验证规则
在 `SendTransaction` 请求的处理过程中，需要对交易进行验证。验证规则如下：
1. **余额充足**: `from` 地址的`balance` 余额必须大于等于 `transfer_amount` + `fee`。
2. **签名有效**: 使用 `from` 地址对应的私钥对交易信息进行签名，签名算法采用 ECDSA，曲线为 secp256k1。
3. **时间戳合理**: `timestamp` 必须在当前时间戳的合理范围内。
4. **哈希正确**: `hash` 必须是交易信息 (包括 `transaction_type`、`from`、`to`、`transfer_amount`、`fee`、`timestamp`、`nonce`、`signature`) 的 SHA256 哈希值。
5. **交易序号正确**: `nonce` 必须与 `from` 地址的当前 `nonce` 值一致。
6. **Merkle Proof 有效**: 

### 8. 转账核心规则
1、验证转账动作是否有效
    1、余额是否足够
    2、地址是否正确
    3、转账是否重复
2、验证成功后，锁定转账金额，创建交易记录，存入交易池，等待确认
3、确认后，将交易记录存入数据库和区块中
4、更新地址余额
5、通知客户端


