use crate::crypto::hash::Hash;
use crate::crypto::signature::SignatureWrapper;
use crate::types::amount::Amount;
use crate::types::block::Block;
use crate::types::transaction::TransactionDetail;
use crate::types::transaction::TransactionType;
use chrono::{DateTime, Utc};
use hex;
use serde::{Deserialize, Serialize};

// 添加自定义序列化函数：将 Option<Vec<u8>> 转换为十六进制字符串或 None
fn serialize_optional_hex<S>(data: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match data {
        // 如果有值，将二进制数据转换为十六进制字符串
        Some(bytes) => serializer.serialize_str(&hex::encode(bytes)),
        // 如果是 None，则序列化为 null
        None => serializer.serialize_none(),
    }
}

// 添加自定义反序列化函数：将十六进制字符串或 null 转换回 Option<Vec<u8>>
fn deserialize_optional_hex<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)?
        .map(|s| hex::decode(s).map_err(serde::de::Error::custom))
        .transpose()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSchema {
    pub address: String,
    pub nonce: u64,
    pub available_balance: Amount,
    pub locked_balance: Amount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSchema {
    pub transaction_hash: Hash,
    pub transaction_type: TransactionType,
    pub from: String,
    pub to: String,
    pub transfer_amount: Amount,
    pub locked: bool,
    pub unlocked_time: i64,
    pub nonce: i64,
    pub initiator_signature: SignatureWrapper,
    pub timestamp: DateTime<Utc>,
    pub fee: Amount,
    pub block_hash: Option<Hash>,
    pub transaction_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockSchema {
    pub block_hash: Hash,
    pub parent_hash: Hash,
    pub height: u64,
    pub timestamp: DateTime<Utc>,
    pub merkle_root: Hash,
    pub validator: String,
    pub signature: SignatureWrapper,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionPoolSchema {
    pub transaction_hash: Hash,
    pub transaction_type: TransactionType,
    pub from_address: String,
    pub to_address: String,
    pub transfer_amount: Amount,
    pub locked: bool,
    pub unlocked_time: i64,
    pub nonce: i64,
    pub initiator_signature: SignatureWrapper,
    pub timestamp: DateTime<Utc>,
    pub fee: Amount,
    pub transaction_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleTreeSchema {
    pub root_hash: Hash,
    pub block_hash: Hash,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProofSchema {
    pub transaction_hash: Hash,
    pub root_hash: Hash,
    pub proof_data: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Created,    // 交易创建
    Validating, // 验证中
    Pending,    // 待处理（验证通过）
    Locked,     // 金额锁定
    Confirmed,  // 已确认（已打包到区块）
    Failed,     // 失败
    Expired,    // 超时
    Rejected,   // 拒绝
}

// 添加 From/Into trait 实现，方便类型转换
impl From<TransactionSchema> for TransactionPoolSchema {
    fn from(transaction: TransactionSchema) -> Self {
        Self {
            transaction_hash: transaction.transaction_hash,
            transaction_type: transaction.transaction_type,
            from_address: transaction.from,
            to_address: transaction.to,
            transfer_amount: transaction.transfer_amount,
            locked: transaction.locked,
            unlocked_time: transaction.unlocked_time,
            nonce: transaction.nonce,
            initiator_signature: transaction.initiator_signature,
            timestamp: transaction.timestamp,
            fee: transaction.fee,
            transaction_status: transaction.transaction_status,
        }
    }
}

impl From<TransactionDetail> for TransactionSchema {
    fn from(transaction: TransactionDetail) -> Self {
        Self {
            transaction_hash: transaction.transaction_hash,
            transaction_type: transaction.transaction_type,
            from: transaction.from,
            to: transaction.to,
            transfer_amount: transaction.transfer_amount,
            locked: transaction.locked,
            unlocked_time: transaction.unlocked_time.timestamp(),
            nonce: transaction.nonce,
            initiator_signature: transaction.initiator_signature,
            timestamp: transaction.timestamp,
            fee: transaction.fee,
            block_hash: None,
            transaction_status: transaction.transaction_status,
        }
    }
}

impl From<Block> for BlockSchema {
    fn from(block: Block) -> Self {
        Self {
            block_hash: block.header.block_hash,
            parent_hash: block.header.parent_hash,
            height: block.header.height,
            timestamp: block.header.timestamp,
            merkle_root: block.header.merkle_root,
            validator: block.header.validator,
            signature: block.header.signature,
        }
    }
}
