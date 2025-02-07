use crate::ledger::block::BlockManager;
use crate::ledger::db::operation::WalletOperations;
use crate::ledger::merkletree::MerkleTreeManager;
use crate::types::ledger::MintingEvent;
use crate::types::transaction::{TransactionDetail, TransactionType};
use crate::types::amount::Amount;
use crate::crypto::signature::SignatureWrapper;
use crate::ledger::minting::minting_error::MintingError;
use log::{debug, info};
use sqlx::SqlitePool;
use std::sync::{Arc, Mutex};
use chrono::Utc;

/// 铸造执行器
pub struct MintingExecutor {
    wallet_ops: WalletOperations,
    block_manager: Arc<BlockManager>,
    merkle_manager: Arc<Mutex<MerkleTreeManager>>,
}

impl MintingExecutor {
    /// 创建新的铸造执行器实例
    pub fn new(
        pool: Arc<SqlitePool>,
        block_manager: Arc<BlockManager>,
        merkle_manager: Arc<Mutex<MerkleTreeManager>>,
    ) -> Self {
        Self {
            wallet_ops: WalletOperations::new(pool),
            block_manager,
            merkle_manager,
        }
    }

    /// 执行铸造操作
    pub async fn execute_minting(&self, minting_event: &MintingEvent) -> Result<(), MintingError> {
        debug!("开始执行铸造操作...");

        // 1. 创建铸造交易
        let transaction = self.create_minting_transaction(minting_event).await?;

        // 2. 更新接收钱包余额
        self.update_recipient_balance(&transaction).await?;

        // 3. 创建新区块并更新默克尔树
        let block = self.block_manager
            .create_block(&[transaction.clone()])
            .await
            .map_err(|e| MintingError::Other(format!("区块创建失败: {}", e)))?;

        // 4. 验证新区块
        if !self.block_manager
        .verify_block(&block)
        .await
        .map_err(|e| MintingError::Other(format!("区块验证失败: {}", e)))? {
        return Err(MintingError::Other("区块验证未通过".to_string()));
        }

        info!(
            "铸造操作执行成功: 接收地址={}, 数量={}, 区块高度={}",
            minting_event.recipient_address.to_string(),
            minting_event.mint_amount.to_string(),
            block.header.height
        );

        Ok(())
    }

    /// 创建铸造交易
    async fn create_minting_transaction(
        &self,
        minting_event: &MintingEvent,
    ) -> Result<TransactionDetail, MintingError> {
        debug!("正在创建铸造交易...");

        let transaction = TransactionDetail {
            transaction_type: TransactionType::Mint,
            from: minting_event.initiator_address.clone(),
            to: minting_event.recipient_address.clone(),
            transfer_amount: minting_event.mint_amount.clone(),
            nonce: 0, // 铸造交易的nonce固定为0
            locked: minting_event.locked,
            unlocked_time: minting_event.unlocked_time,
            initiator_signature: SignatureWrapper::from_hex_string(&minting_event.transaction_hash.to_string())
                .map_err(|e| MintingError::SignatureError(e.to_string()))?,
            timestamp: Utc::now(),
            fee: Amount::from_biguint(num_bigint::BigUint::from(0u64))
                .map_err(|e| MintingError::Other(format!("铸造手续费错误: {}", e)))?,
            transaction_hash: minting_event.transaction_hash.clone(),
            transaction_status: "confirmed".to_string(),

        };

        debug!("铸造交易创建成功: {:?}", transaction);
        Ok(transaction)

    }

    /// 更新接收钱包余额
    async fn update_recipient_balance(&self, transaction: &TransactionDetail) -> Result<(), MintingError> {
        debug!("正在更新接收钱包余额...");

        // 获取当前余额
        let mut wallet = self.wallet_ops
            .get_wallet(&transaction.to)
            .await
            .map_err(|e| MintingError::Other(format!("钱包查询失败: {}", e)))?
            .ok_or_else(|| MintingError::InvalidMintingFormat(
                "接收钱包不存在".to_string(),
            ))?;


        // 更新余额
        wallet.balance = (wallet.balance + transaction.transfer_amount.clone())
            .map_err(|e| MintingError::Other(format!("余额计算错误: {}", e)))?;


        // 保存更新后的余额
        self.wallet_ops
            .update_wallet_balance(&transaction.to, wallet.balance)
            .await
            .map_err(|e| MintingError::Other(format!("钱包余额更新失败: {}", e)))?;


        debug!("接收钱包余额更新成功");
        Ok(())
    }
}