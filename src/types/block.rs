use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use crate::types::transaction::Transaction;

// 区块头
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockHeader {
    pub parent_hash: Vec<u8>,
    pub height: u64,
    pub timestamp: DateTime<Utc>,
    #[serde(with = "hex")]
    pub merkle_root: Vec<u8>,
    pub validator: String,
    pub signature: Vec<u8>,
    #[serde(with = "hex")]
    pub block_hash: Vec<u8>,
    pub block_number: u64,
    pub previous_block_hash: Vec<u8>,
}

// 区块
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Default for BlockHeader {
    fn default() -> Self {
        BlockHeader {
            parent_hash: Vec::new(),
            height: 0,
            timestamp: Utc::now(),
            merkle_root: Vec::new(),
            validator: String::new(),
            signature: Vec::new(),
            block_hash: Vec::new(),
            block_number: 0,
            previous_block_hash: Vec::new(),
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        Block {
            header: BlockHeader::default(),
            transactions: Vec::new(),
        }
    }
}

impl AsRef<[u8]> for Block {
    fn as_ref(&self) -> &[u8] {
        let serialized = serde_json::to_vec(self).unwrap();
        let boxed_slice = serialized.into_boxed_slice();
        Box::leak(boxed_slice)
    }
}

impl PartialOrd for Block {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Block {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.header.block_hash.cmp(&other.header.block_hash)
    }
}

// 计算区块哈希的函数
pub fn calculate_block_hash(header: &BlockHeader) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(&header.parent_hash);
    hasher.update(header.height.to_be_bytes());
    hasher.update(header.timestamp.to_rfc3339().as_bytes());
    hasher.update(&header.merkle_root);
    hasher.update(&header.validator);
    hasher.update(&header.signature);
    hasher.finalize().to_vec()
}