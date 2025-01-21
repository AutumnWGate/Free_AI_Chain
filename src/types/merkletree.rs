use crate::types::transaction::Transaction;
use crate::types::block::Block;
use merkletree::merkle::Element;

// 为 Transaction 实现 Element trait
impl Element for Transaction {
    fn byte_len() -> usize {
        // 使用动态计算的方式，而不是固定大小
        let sample = Transaction::default();
        serde_json::to_vec(&sample).unwrap().len()
    }

    fn from_slice(bytes: &[u8]) -> Self {
        match serde_json::from_slice(bytes) {
            Ok(transaction) => transaction,
            Err(_) => Transaction::default() // 提供一个默认值而不是 panic
        }
    }

    fn copy_to_slice(&self, bytes: &mut [u8]) {
        let serialized = serde_json::to_vec(self).unwrap_or_default();
        let len = serialized.len().min(bytes.len());
        if len > 0 {
            bytes[..len].copy_from_slice(&serialized[..len]);
            // 如果目标缓冲区更大，用零填充剩余部分
            if bytes.len() > len {
                bytes[len..].fill(0);
            }
        }
    }
}

// 为 Block 实现 Element trait
impl Element for Block {
    fn byte_len() -> usize {
        // 使用动态计算的方式
        let sample = Block::default();
        serde_json::to_vec(&sample).unwrap().len()
    }

    fn from_slice(bytes: &[u8]) -> Self {
        match serde_json::from_slice(bytes) {
            Ok(block) => block,
            Err(_) => Block::default()
        }
    }

    fn copy_to_slice(&self, bytes: &mut [u8]) {
        let serialized = serde_json::to_vec(self).unwrap_or_default();
        let len = serialized.len().min(bytes.len());
        if len > 0 {
            bytes[..len].copy_from_slice(&serialized[..len]);
            if bytes.len() > len {
                bytes[len..].fill(0);
            }
        }
    }
}