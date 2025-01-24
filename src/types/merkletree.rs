
use crate::types::block::Block;
use merkletree::merkle::Element;

// 为 Block 实现 Element trait
impl Element for Block {
    fn byte_len() -> usize {
        let sample = Block::default();
        serde_json::to_vec(&sample)
            .map_err(MerkleTreeError::SerializationError)
            .unwrap_or_default()
            .len()
    }

    fn from_slice(bytes: &[u8]) -> Self {
        serde_json::from_slice(bytes)
            .unwrap_or_else(|_| Block::default())
    }

    fn copy_to_slice(&self, bytes: &mut [u8]) {
        let serialized = serde_json::to_vec(self)
            .map_err(MerkleTreeError::SerializationError)
            .unwrap_or_default();
        let len = serialized.len().min(bytes.len());
        if len > 0 {
            bytes[..len].copy_from_slice(&serialized[..len]);
            if bytes.len() > len {
                bytes[len..].fill(0);
            }
        }
    }
}

#[derive(Debug)]
pub enum MerkleTreeError {
    SerializationError(serde_json::Error),
    DeserializationError(serde_json::Error),
}

impl std::fmt::Display for MerkleTreeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MerkleTreeError::SerializationError(e) => write!(f, "序列化错误: {}", e),
            MerkleTreeError::DeserializationError(e) => write!(f, "反序列化错误: {}", e),
        }
    }
}