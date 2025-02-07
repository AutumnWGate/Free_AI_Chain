use crate::ledger::minting::minting_error::MintingError;
use crate::ledger::minting::minting_error::WhitelistError;

const ADMIN_WALLET_ADDRESS: &str = "YOUR_ADMIN_WALLET_ADDRESS";  //  请替换为实际的管理员钱包地址

/// 白名单管理器 (MVP001 简化版本)
pub struct MintingWhitelist {}

impl MintingWhitelist {
    /// 创建新的白名单管理器实例
    pub fn new() -> Self {
        MintingWhitelist {}
    }

    /// 检查地址是否是管理员白名单地址
    pub fn is_admin_whitelisted(&self, address: &str) -> Result<bool, MintingError> {
        if address == ADMIN_WALLET_ADDRESS {
            Ok(true)
        } else {
            Err(MintingError::WhitelistError(WhitelistError::AddressNotWhitelisted(address.to_string())))
        }
    }


}
