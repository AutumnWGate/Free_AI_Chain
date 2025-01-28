use hex::FromHex;
use merkletree::hash::Algorithm;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};
use std::hash::Hasher;

#[derive(Debug, thiserror::Error)]
pub enum HashError {
    #[error("无效的哈希长度")]
    InvalidLength,
    #[error("哈希计算错误")]
    ComputationError,
}

/// 固定长度的哈希类型包装器
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hash(pub(crate) [u8; 32]);

impl Hash {
    /// 创建一个新的空哈希
    pub fn new() -> Self {
        Hash([0u8; 32])
    }

    /// 从字节切片创建哈希
    pub fn from_slice(slice: &[u8]) -> Result<Self, &'static str> {
        if slice.len() != 32 {
            return Err("Invalid hash length");
        }
        let mut hash = [0u8; 32];
        hash.copy_from_slice(slice);
        Ok(Hash(hash))
    }

    /// 获取底层字节数组
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Serialize for Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex = hex::encode(self.0);
        serializer.serialize_str(&hex)
    }
}

impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_str = String::deserialize(deserializer)?;
        let bytes = Vec::from_hex(&hex_str).map_err(serde::de::Error::custom)?;

        Self::from_slice(&bytes).map_err(serde::de::Error::custom)
    }
}

/// 计算数据的 SHA-256 哈希值
pub fn sha256(data: &[u8]) -> Result<Hash, HashError> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    Hash::from_slice(&hasher.finalize()).map_err(|_| HashError::ComputationError)
}

/// 计算多个数据片段的组合哈希值
pub fn sha256_concat(data_pieces: &[&[u8]]) -> Result<Hash, HashError> {
    let mut hasher = Sha256::new();
    for piece in data_pieces {
        hasher.update(piece);
    }
    Hash::from_slice(&hasher.finalize()).map_err(|_| HashError::ComputationError)
}

/// 将 Vec<u8> 转换为哈希
pub fn to_hash(hash: Vec<u8>) -> Result<Hash, &'static str> {
    Hash::from_slice(&hash)
}

impl std::str::FromStr for Hash {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(|_| "Invalid hex string")?;
        Self::from_slice(&bytes)
    }
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl Hasher for Hash {
    fn finish(&self) -> u64 {
        let mut result = 0u64;
        for chunk in self.0.chunks(8) {
            let mut val = 0u64;
            for &byte in chunk {
                val = (val << 8) | u64::from(byte);
            }
            result ^= val;
        }
        result
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        self.0.copy_from_slice(&hasher.finalize());
    }
}

impl Algorithm<[u8; 32]> for Hash {
    fn hash(&mut self) -> [u8; 32] {
        self.0
    }
}
