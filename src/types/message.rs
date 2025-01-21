use serde::{Deserialize, Serialize};
use crate::types::block::Block;
use crate::types::node::NodeInfo;
use crate::types::amount::Amount;
use crate::types::transaction::Transaction;

/// 消息类型
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum MessageType {
    Request,
    Response,
    Heartbeat,
    Data,
}

/// 请求
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Request {
    GetNodeInfo,
    GetBalance { address: String },
    SendTransaction { transaction: Transaction },
    GetMerkleProof { transaction_hash: Vec<u8>, block_hash: Vec<u8> },
    GetBlock { block_hash: Vec<u8> },
    GetBlockByHeight { height: u64 },
    GetLatestBlock,
}

/// 响应
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Response {
    GetNodeInfoResponse(NodeInfo),
    GetBalanceResponse { balance: Amount },
    SendTransactionResponse {#[serde(with = "serde_bytes")] transaction_hash: Vec<u8> },
    GetMerkleProofResponse { merkle_proof: Vec<Vec<u8>> },
    GetBlockResponse { block: Block },
    GetLatestBlockResponse { block: Block },
    GetBlockByHeightResponse { block: Block },
    Error { message: String },
}