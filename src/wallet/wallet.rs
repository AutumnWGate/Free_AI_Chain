use crate::types::amount::Amount;
use crate::wallet::address::WalletAddress;
use crate::wallet::error::{WalletError, KeyManagerError};
use crate::wallet::key_manager::KeyManager;
use crate::wallet::mnemonic;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 钱包结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    /// 钱包地址
    pub address: WalletAddress,
    /// 钱包余额
    pub balance: Amount,
    /// 交易计数器
    pub nonce: u64,
    /// 加密后的助记词 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted_mnemonic: Option<Vec<u8>>,
    /// 密钥管理器 (运行时)
    #[serde(skip)]
    key_manager: Option<Arc<KeyManager>>,
}

impl Wallet {
    /// 创建新钱包
    /// 
    /// # Arguments
    /// * `password` - 用户密码，用于加密助记词
    /// 
    /// # Returns
    /// * `Result<(Self, String), WalletError>` - 返回钱包实例和明文助记词
    pub fn create_wallet(password: &str) -> Result<(Self, String), WalletError> {
        debug!("正在创建新钱包");
        
        // 验证密码复杂度
        Self::validate_password(password)?;
        
        // 使用密码生成助记词
        let mnemonic = mnemonic::generate_from_password(password)
            .map_err(|e| {
                error!("生成助记词失败: {}", e);
                WalletError::MnemonicGeneration(e.to_string())
            })?;
        
        // 创建密钥管理器
        let key_manager = Arc::new(KeyManager::from_mnemonic(&mnemonic, password)?);
        
        // 使用助记词生成地址
        let wallet_address = WalletAddress::new_segwit_address(&mnemonic)?;
        
        // 加密助记词
        let encrypted_mnemonic = Self::encrypt_mnemonic(&mnemonic, password)?;
        
        let wallet = Wallet {
            address: wallet_address,
            balance: Amount::default(),
            nonce: 0,
            encrypted_mnemonic: Some(encrypted_mnemonic),
            key_manager: Some(key_manager),
        };
        
        info!("新钱包创建成功，地址: {}", wallet.address.to_string());
        Ok((wallet, mnemonic))
    }

    /// 从助记词恢复钱包
    pub fn from_mnemonic(mnemonic: &str, password: &str) -> Result<Self, WalletError> {
        debug!("正在从助记词恢复钱包");
        
        // 验证密码复杂度
        Self::validate_password(password)?;
        
        // 创建密钥管理器
        let key_manager = Arc::new(KeyManager::from_mnemonic(mnemonic, password)?);
        
        // 验证助记词并生成地址
        let wallet_address = WalletAddress::new_segwit_address(mnemonic)?;
        
        // 加密助记词
        let encrypted_mnemonic = Self::encrypt_mnemonic(mnemonic, password)?;
        
        let wallet = Wallet {
            address: wallet_address,
            balance: Amount::default(),
            nonce: 0,
            encrypted_mnemonic: Some(encrypted_mnemonic),
            key_manager: Some(key_manager),
        };
        
        info!("钱包恢复成功，地址: {}", wallet.address.to_string());
        Ok(wallet)
    }

    /// 使用密码找回助记词
    pub fn recover_mnemonic(&self, password: &str) -> Result<String, WalletError> {
        debug!("正在尝试找回助记词");
        
        let encrypted_mnemonic = self.encrypted_mnemonic.as_ref()
            .ok_or_else(|| {
                error!("找不到加密的助记词");
                WalletError::NoMnemonic
            })?;
            
        Self::decrypt_mnemonic(encrypted_mnemonic, password)
    }

    /// 签名交易
    pub fn sign_transaction(&self, message: &[u8]) -> Result<Vec<u8>, WalletError> {
        debug!("正在签名交易");
        
        let key_manager = self.key_manager.as_ref()
            .ok_or_else(|| KeyManagerError::NoKeyManager)?;
            
        key_manager.sign_message(message)
            .map_err(|e| WalletError::SignatureError(e.to_string()))
            .map(|sig| sig.to_bytes().to_vec())
    }

    /// 验证密码复杂度
    fn validate_password(password: &str) -> Result<(), WalletError> {
        if password.len() < 8 {
            warn!("密码长度不足8位");
            return Err(WalletError::InvalidPassword("密码长度不能少于8位".to_string()));
        }
        
        if !password.chars().any(|c| c.is_uppercase()) {
            warn!("密码未包含大写字母");
            return Err(WalletError::InvalidPassword("密码必须包含至少一个大写字母".to_string()));
        }
        
        if !password.chars().any(|c| c.is_lowercase()) {
            warn!("密码未包含小写字母");
            return Err(WalletError::InvalidPassword("密码必须包含至少一个小写字母".to_string()));
        }
        
        if !password.chars().any(|c| c.is_numeric()) {
            warn!("密码未包含数字");
            return Err(WalletError::InvalidPassword("密码必须包含至少一个数字".to_string()));
        }
        
        Ok(())
    }

    /// 加密助记词
    fn encrypt_mnemonic(mnemonic: &str, password: &str) -> Result<Vec<u8>, WalletError> {
        mnemonic::encrypt_mnemonic(mnemonic, password)
            .map_err(|e| WalletError::EncryptionError(e.to_string()))
    }

    /// 解密助记词
    fn decrypt_mnemonic(encrypted: &[u8], password: &str) -> Result<String, WalletError> {
        mnemonic::decrypt_mnemonic(encrypted, password)
            .map_err(|e| WalletError::DecryptionError(e.to_string()))
    }

    /// 获取钱包地址
    pub fn address(&self) -> &WalletAddress {
        &self.address
    }

    /// 获取钱包余额
    pub fn get_balance(&self) -> Amount {
        self.balance.clone()
    }

    /// 获取交易计数器
    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    /// 更新钱包余额
    pub fn update_balance(&mut self, new_balance: Amount) {
        debug!("更新钱包余额: {:?} -> {:?}", self.balance, new_balance);
        self.balance = new_balance;
    }

    /// 增加交易计数器
    pub fn increment_nonce(&mut self) {
        self.nonce += 1;
        debug!("增加交易计数器: {}", self.nonce);
    }
}
