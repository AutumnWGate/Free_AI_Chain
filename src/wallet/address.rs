use crate::crypto::address::Address as CryptoAddress;
use crate::wallet::error::WalletError;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAddress(CryptoAddress);

impl WalletAddress {
    /// 从助记词生成 BIP44 地址
    pub fn new_from_mnemonic(mnemonic: &str) -> Result<Self, WalletError> {
        debug!("正在从助记词生成 BIP44 地址");
        CryptoAddress::new_from_mnemonic(mnemonic)
            .map(|addr| {
                info!("BIP44 地址生成成功");
                WalletAddress(addr)
            })
            .map_err(|e| {
                error!("BIP44 地址生成失败: {}", e);
                WalletError::Mnemonic(e.to_string())
            })
    }

    /// 生成 SegWit 地址 (BIP84)
    pub fn new_segwit_address(mnemonic: &str) -> Result<Self, WalletError> {
        debug!("正在生成 SegWit 地址");
        CryptoAddress::new_segwit_address(mnemonic)
            .map(|addr| {
                info!("SegWit 地址生成成功");
                WalletAddress(addr)
            })
            .map_err(|e| {
                error!("SegWit 地址生成失败: {}", e);
                WalletError::SegwitAddressError(e.to_string())
            })
    }
    /// 生成支付码 (BIP47)
    pub fn generate_payment_code(mnemonic: &str) -> Result<String, WalletError> {
        debug!("正在生成 BIP47 支付码");
        CryptoAddress::generate_payment_code(mnemonic)
            .map(|code| {
                info!("支付码生成成功");
                code
            })
            .map_err(|e| {
                error!("支付码生成失败: {}", e);
                WalletError::PaymentCode(e.to_string())
            })
    }

    /// 验证地址格式
    pub fn validate_address(address: &str) -> bool {
        debug!("正在验证地址格式: {}", address);
        
        if address.is_empty() {
            warn!("地址为空");
            return false;
        }
        
        if !address.starts_with("faic") {
            warn!("地址前缀错误，应为 'faic'");
            return false;
        }

        if address.len() != 42 {
            warn!("地址长度错误，应为 42 字节，实际为 {}", address.len());
            return false;
        }

        match CryptoAddress::from_string(address) {
            Ok(addr) => {
                let is_valid = addr.to_string().len() == 40;
                if is_valid {
                    debug!("地址验证成功");
                } else {
                    warn!("地址格式无效");
                }
                is_valid
            },
            Err(e) => {
                error!("地址解码失败: {}", e);
                false
            }
        }
    }

    /// 获取地址字符串
    pub fn to_string(&self) -> String {
        debug!("正在转换地址为字符串");
        self.0.to_string()
    }

    /// 从字符串创建地址
    pub fn from_string(address: &str) -> Result<Self, WalletError> {
        debug!("正在从字符串创建地址: {}", address);
        CryptoAddress::from_string(address)
            .map(|addr| {
                info!("地址创建成功");
                WalletAddress(addr)
            })
            .map_err(|e| {
                error!("地址创建失败: {}", e);
                WalletError::AddressError(e.to_string())
            })
    }
}

impl AsRef<[u8]> for WalletAddress {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl fmt::Display for WalletAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}