use crate::crypto::hash::Hash;
use crate::types::amount::Amount;
use crate::types::block::Block;
use crate::types::node::NodeInfo;
use crate::types::transaction::Transaction;
use serde::{Deserialize, Serialize};

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
    GetBalance {
        address: String,
    },
    SendTransaction {
        transaction: Transaction,
    },
    GetMerkleProof {
        transaction_hash: Hash,
        block_hash: Hash,
    },
    GetBlock {
        block_hash: Hash,
    },
    GetBlockByHeight {
        height: u64,
    },
    GetLatestBlock,
}

/// 响应
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Response {
    GetNodeInfoResponse(NodeInfo),
    GetBalanceResponse { balance: Amount },
    SendTransactionResponse { transaction_hash: Hash },
    GetMerkleProofResponse { merkle_proof: Vec<Hash> },
    GetBlockResponse { block: Block },
    GetLatestBlockResponse { block: Block },
    GetBlockByHeightResponse { block: Block },
    Error { message: String },
}
