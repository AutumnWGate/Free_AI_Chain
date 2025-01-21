use serde::{Deserialize, Serialize};
use crate::types::amount::Amount;
// 钱包记录
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wallet {
    pub address: String,
    pub balance: Amount,
    pub nonce: u64,
}

impl Wallet {
    pub fn new() -> Self {
        let address = Address::new().to_string(); // 使用 Address 类型生成地址
        Wallet {
            address,
            balance: Amount::default(), // 初始余额为 0
            nonce: 0,
        }
    }
}