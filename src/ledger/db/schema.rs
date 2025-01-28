use crate::crypto::hash::Hash;
use crate::crypto::signature::SignatureWrapper;
use crate::types::amount::Amount;
use crate::types::block::Block;
use crate::types::transaction::Transaction;
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
    pub balance: Amount,
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSchema {
    pub transaction_hash: Hash,
    pub transaction_type: TransactionType,
    pub from: String,
    pub to: String,
    pub transfer_amount: Amount,
    pub nonce: u64,
    pub signature: SignatureWrapper,
    pub timestamp: DateTime<Utc>,
    pub fee: Amount,
    pub block_hash: Option<Hash>,
    pub status: TransactionStatus,
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
    pub hash: Hash,
    pub transaction_type: TransactionType,
    pub from_address: String,
    pub to_address: String,
    pub transfer_amount: Amount,
    pub nonce: u64,
    pub signature: SignatureWrapper,
    pub timestamp: DateTime<Utc>,
    pub fee: Amount,
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
    Pending,
    Confirmed,
    Failed,
}

// 添加 From/Into trait 实现，方便类型转换
impl From<TransactionSchema> for TransactionPoolSchema {
    fn from(transaction: TransactionSchema) -> Self {
        Self {
            hash: transaction.transaction_hash,
            transaction_type: transaction.transaction_type,
            from_address: transaction.from,
            to_address: transaction.to,
            transfer_amount: transaction.transfer_amount,
            nonce: transaction.nonce,
            signature: transaction.signature,
            timestamp: transaction.timestamp,
            fee: transaction.fee,
        }
    }
}

impl From<Transaction> for TransactionSchema {
    fn from(transaction: Transaction) -> Self {
        Self {
            transaction_hash: transaction.transaction_hash,
            transaction_type: transaction.transaction_type,
            from: transaction.from,
            to: transaction.to,
            transfer_amount: transaction.transfer_amount,
            nonce: transaction.nonce,
            signature: transaction.signature,
            timestamp: transaction.timestamp,
            fee: transaction.fee,
            block_hash: None,
            status: TransactionStatus::Pending,
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
