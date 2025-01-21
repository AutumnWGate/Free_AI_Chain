use serde::{Deserialize, Serialize};
use crate::crypto::hash::sha256;
use base58::{ToBase58, FromBase58};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Address(Vec<u8>);

impl Address {
    pub fn new() -> Self {
        // 1. 生成一个随机数（在实际应用中，这通常是一个私钥）
        let random_bytes: [u8; 32] = rand::random(); // 使用 rand crate 生成随机字节

        // 2. 计算随机数的 SHA-256 哈希值
        let hash = sha256(&random_bytes);

        // 3. (可选) 添加版本前缀或其他标识符

        // 4. 对哈希值进行 Base58 编码
        Address(hash)
    }

    pub fn to_string(&self) -> String {
        self.0.to_base58()
    }

    pub fn from_string(encoded: &str) -> Result<Self, &'static str> {
        match encoded.from_base58() {
            Ok(decoded) => Ok(Address(decoded)),
            Err(_) => Err("Invalid Base58 encoding"),
        }
    }
}