use crate::ledger::minting::minting_error::MintingError;
use crate::ledger::minting::minting_whitelist::MintingWhitelist;
use crate::types::ledger::MintingEvent;
use log::{debug, error, info};
use crate::crypto::signature::SignatureWrapper;
use crate::types::amount::Amount;
use crate::wallet::address::WalletAddress;
use crate::ledger::db::operation::WalletOperations;
use sqlx::SqlitePool;
use std::sync::Arc;




/// 铸造验证器
pub struct MintingValidator {
    whitelist: MintingWhitelist,
    wallet_ops: WalletOperations,
}

impl MintingValidator {
    /// 创建新的铸造验证器实例
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        MintingValidator {
            whitelist: MintingWhitelist::new(),
            wallet_ops: WalletOperations::new(pool),
        }
    }


    /// 验证铸造请求
    pub async fn validate_minting(&self, minting_event: &MintingEvent) -> Result<bool, MintingError> {
        debug!("开始验证铸造请求...");

        // 1. 获取并验证铸造发起者地址
        let initiator_address = &minting_event.initiator_address;
        debug!("铸造发起者地址: {}", initiator_address);

        // 2. 验证白名单
        match self.whitelist.is_admin_whitelisted(initiator_address) {
            Ok(true) => {
                debug!("白名单验证通过");
            }
            Ok(false) | Err(_) => {
                error!("铸造发起者不在白名单中: {}", initiator_address);
                return Ok(false);
            }
        }

        // 3. 验证签名
        let signature = SignatureWrapper::from_hex_string(&minting_event.transaction_hash.to_string())
            .map_err(|e| MintingError::SignatureError(e.to_string()))?;

        // 序列化铸造事件数据用于签名验证
        let event_data = serde_json::to_vec(&minting_event)
            .map_err(|e| MintingError::SerializationError(e))?;

        if !signature.verify(&event_data, initiator_address)
            .map_err(|e| MintingError::SignatureError(e.to_string()))? {
            error!("铸造请求签名验证失败");
            return Ok(false);
        }
        debug!("签名验证通过");

        // 4. 验证铸造信息格式
        if !self.validate_minting_format(minting_event).await? {
            error!("铸造信息格式验证失败");
            return Ok(false);
        }
        debug!("铸造信息格式验证通过");

        info!("铸造请求验证成功: initiator={}", initiator_address);
        Ok(true)
    }

    /// 验证铸造信息格式
    async fn validate_minting_format(&self, minting_event: &MintingEvent) -> Result<bool, MintingError> {
        // 验证接收地址不为空
        if minting_event.recipient_address.is_empty() {
            return Err(MintingError::InvalidMintingFormat(

                "接收地址不能为空".to_string(),
            ));
        }

        // 验证接收地址是否是有效的钱包地址
        if !WalletAddress::validate_address(&minting_event.recipient_address) {
            return Err(MintingError::InvalidMintingFormat(
                "接收地址格式无效".to_string(),
            ));
        }

        // 验证接收钱包是否存在
        match self.wallet_ops.get_wallet(&minting_event.recipient_address).await {
            Ok(Some(_)) => {
                debug!("接收钱包验证通过");
            }
            Ok(None) => {
                return Err(MintingError::InvalidMintingFormat(
                    "接收钱包地址不存在".to_string(),
                ));
            }
            Err(e) => {
                return Err(MintingError::Other(format!("数据库查询错误: {}", e)));
            }
        }


        // 验证铸造数量大于0
        if minting_event.mint_amount == Amount::from_biguint(num_bigint::BigUint::from(0u64))

            .map_err(|e| MintingError::InvalidMintingAmount(e.to_string()))? 
        {
            return Err(MintingError::InvalidMintingAmount(
                "铸造数量必须大于0".to_string(),
            ));
        }

        // 验证时间戳
        let now = chrono::Utc::now();
        if minting_event.timestamp > now {
            return Err(MintingError::InvalidMintingFormat(
                "时间戳不能超过当前时间".to_string(),
            ));
        }

        // 如果是锁定的代币,验证解锁时间
        if minting_event.locked && minting_event.unlocked_time <= now {
            return Err(MintingError::InvalidMintingFormat(
                "锁定代币的解锁时间必须大于当前时间".to_string(),
            ));
        }

        Ok(true)
    }
}
