use crate::{crypto::hash::Hash, types::block::Block};
use crate::types::transaction::TransactionDetail;
use crate::types::transaction::TransactionType;
use crate::types::wallet::Wallet;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

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
    pub pending_transactions: Vec<TransactionDetail>,
    pub confirmed_transactions: Vec<TransactionDetail>,
    pub block_hash: Option<Vec<u8>>, // 所属区块哈希
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
    DeleteWallet { address: String },
    GetBalance { address: String },
    GetTransactionHistory { address: String },
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

/// 铸造权限类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MintingPermissionType {
    Admin,
    SmartContract,
    DApp,
}

/// 铸造权限配置（对应铸造权限设计文档2.2节）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintingWhitelistConfig {
    pub admin_whitelist: Vec<String>, // 管理员地址白名单
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub smart_contract_whitelist: Vec<String>, // 智能合约白名单（MVP阶段留空）
    #[serde(skip_serializing_if = "Vec::is_empty", default)] 
    pub dapp_whitelist: Vec<String>, // DApp白名单（MVP阶段留空）
}

/// 铸造事件记录（对应minting模块的铸造历史记录要求）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintingEvent {
    pub transaction_hash: Hash,
    pub transaction_type: TransactionType,
    pub block_hash: Hash,
    pub block_height: u64,
    pub initiator_address: String,
    pub role_type: MintingPermissionType,
    pub recipient_address: String,
    pub mint_amount: crate::types::amount::Amount,
    pub locked: bool,
    pub unlocked_time: DateTime<Utc>,
    pub timestamp: DateTime<Utc>,
    pub merkle_proof: Vec<[u8; 32]>, // 证明路径使用标准32字节数组
}