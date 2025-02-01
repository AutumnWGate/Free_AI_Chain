use crate::crypto::address::Address;
use crate::types::amount::Amount;
use serde::{Deserialize, Serialize};

// 钱包记录
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wallet {
    pub address: String,
    pub balance: Amount,
    pub nonce: i64,
}

impl Wallet {
    /// 创建新钱包
    ///
    /// 返回钱包实例和对应的助记词
    pub fn create_wallet() -> Result<(Self, String), &'static str> {
        // 使用默认密码生成助记词
        let password = "default_secure_password"; // 建议从配置或参数传入 TODO
        let mnemonic = Address::generate_mnemonic_from_password(password)?;

        // 使用助记词生成地址
        let address = Address::new_segwit_address(&mnemonic)?;

        Ok((
            Wallet {
                address: address.to_string(),
                balance: Amount::default(),
                nonce: 0,
            },
            mnemonic,
        ))
    }

    /// 从助记词恢复钱包
    pub fn from_mnemonic(mnemonic: &str) -> Result<Self, &'static str> {
        let address = Address::new_segwit_address(mnemonic)?;

        Ok(Wallet {
            address: address.to_string(),
            balance: Amount::default(),
            nonce: 0,
        })
    }
}
