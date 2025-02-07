use super::minting_error::MintingError;
use super::minting_executor::MintingExecutor;
use crate::ledger::validator::minting_validator::MintingValidator;
use crate::ledger::block::BlockManager;
use crate::ledger::merkletree::MerkleTreeManager;
use crate::types::ledger::MintingEvent;
use log::{debug, error, info};
use sqlx::SqlitePool;
use std::sync::{Arc, Mutex};

/// 铸造管理器
pub struct MintingManager {
    validator: MintingValidator,
    executor: MintingExecutor,
}

impl MintingManager {
    /// 创建新的铸造管理器实例
    pub fn new(
        pool: Arc<SqlitePool>,
        block_manager: Arc<BlockManager>,
        merkle_manager: Arc<Mutex<MerkleTreeManager>>,
    ) -> Self {
        Self {
            validator: MintingValidator::new(pool.clone()),
            executor: MintingExecutor::new(pool, block_manager, merkle_manager),
        }
    }

    /// 处理铸造请求
    pub async fn process_minting(&self, minting_event: MintingEvent) -> Result<(), MintingError> {
        debug!("开始处理铸造请求...");

        // 1. 验证铸造请求
        if !self.validator.validate_minting(&minting_event).await? {
            error!("铸造请求验证失败");
            return Err(MintingError::PermissionDenied);
        }
        debug!("铸造请求验证通过");

        // 2. 执行铸造操作
        self.executor.execute_minting(&minting_event).await?;

        info!(
            "铸造请求处理成功: 接收地址={}, 数量={}",
            minting_event.recipient_address.to_string(),
            minting_event.mint_amount.to_string()
        );

        Ok(())
    }

    /// 批量处理铸造请求
    pub async fn batch_process_minting(
        &self,
        minting_events: Vec<MintingEvent>,
    ) -> Result<Vec<Result<(), MintingError>>, MintingError> {
        debug!("开始批量处理铸造请求，数量: {}", minting_events.len());

        let mut results = Vec::with_capacity(minting_events.len());

        for event in minting_events {
            let result = self.process_minting(event).await;
            match &result {
                Ok(_) => debug!("铸造请求处理成功"),
                Err(e) => error!("铸造请求处理失败: {}", e),
            }
            results.push(result);
        }

        debug!("批量铸造请求处理完成");
        Ok(results)
    }

    /// 获取铸造验证器引用
    pub fn get_validator(&self) -> &MintingValidator {
        &self.validator
    }

    /// 获取铸造执行器引用
    pub fn get_executor(&self) -> &MintingExecutor {
        &self.executor
    }
}