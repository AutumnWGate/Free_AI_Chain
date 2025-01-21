use serde::{Deserialize, Serialize};
use crate::types::wallet::Wallet;
use crate::types::block::Block;
use crate::types::transaction::Transaction;

/// 钱包管理相关数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletManagement {
    pub wallet_records: Vec<Wallet>,
    pub actions: Vec<WalletAction>,
}

/// 区块管理相关数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockManagement {
    pub blocks: Vec<Block>,
    pub latest_block: Option<Block>,
    pub block_height: u64,
}

/// 交易管理相关数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionManagement {
    pub pending_transactions: Vec<Transaction>,
    pub confirmed_transactions: Vec<Transaction>,
    pub block_hash: Option<Vec<u8>>,  // 所属区块哈希
}

/// 账本状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerState {
    pub wallet_management: WalletManagement,
    pub block_management: BlockManagement,
    pub transaction_management: TransactionManagement,
}

/// 钱包操作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalletAction {
    CreateWallet,
    DeleteWallet,
    GetBalance,
    GetTransactionHistory,
}

/// 交易验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionVerifyResult {
    Valid,
    InvalidBalance,
    InvalidSignature,
    InvalidNonce,
    InvalidTimestamp,
    InvalidHash,
}

/// 区块验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockVerifyResult {
    Valid,
    InvalidParentHash,
    InvalidMerkleRoot,
    InvalidSignature,
    InvalidTransactions,
}