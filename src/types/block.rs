use crate::crypto::hash::{sha256_concat, Hash};
use crate::crypto::signature::SignatureWrapper;
use crate::types::transaction::TransactionDetail;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

#[derive(Debug, thiserror::Error)]
pub enum BlockError {
    #[error("哈希错误: {0}")]
    HashError(#[from] crate::crypto::hash::HashError),
}

// 区块头
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockHeader {
    pub parent_hash: Hash,
    pub height: u64,
    pub timestamp: DateTime<Utc>,
    pub merkle_root: Hash,
    pub validator: String,
    pub signature: SignatureWrapper,
    pub block_hash: Hash,
    pub block_number: u64,
    pub previous_block_hash: Hash,
}

// 区块
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<TransactionDetail>,
}

impl Default for BlockHeader {
    fn default() -> Self {
        BlockHeader {
            parent_hash: Hash::new(),
            height: 0,
            timestamp: Utc::now(),
            merkle_root: Hash::new(),
            validator: String::new(),
            signature: SignatureWrapper::from_bytes(&[0u8; 65]).expect("默认签名应该总是有效的"), // 这里使用 expect 更合适，因为这是默认值
            block_hash: Hash::new(),
            block_number: 0,
            previous_block_hash: Hash::new(),
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

// 使用线程局部存储来处理 AsRef<[u8]> 实现
thread_local! {
    static BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::new());
}

impl AsRef<[u8]> for Block {
    fn as_ref(&self) -> &[u8] {
        BUFFER.with(|buffer| {
            let mut buf = buffer.borrow_mut();
            *buf = serde_json::to_vec(self).unwrap_or_default();
            // 安全：buf 的生命周期与 BUFFER 绑定
            unsafe { std::mem::transmute(buf.as_slice()) }
        })
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
pub fn calculate_block_hash(header: &BlockHeader) -> Result<Hash, BlockError> {
    sha256_concat(&[
        header.parent_hash.as_ref(),
        &header.height.to_be_bytes(),
        header.timestamp.to_rfc3339().as_bytes(),
        header.merkle_root.as_ref(),
        header.validator.as_bytes(),
        &header.signature.to_bytes().as_ref(),
    ])
    .map_err(BlockError::HashError)
}
