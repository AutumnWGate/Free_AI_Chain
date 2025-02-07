use super::db::operation::TransactionOperations;
use super::db::operation::WalletOperations;
use super::error::LedgerError;
use super::merkletree::MerkleTreeManager;
use crate::crypto::hash::to_hash;
use crate::crypto::hash::Hash;
use crate::crypto::signature::SignatureWrapper;
use crate::types::amount::Amount;
use crate::types::ledger::TransactionManagement;
use crate::types::transaction::TransactionDetail;
use log::{debug, error};
use sqlx::SqlitePool;
use std::sync::Arc;

/// 交易管理器
pub struct TransactionManager {
    db_ops: TransactionOperations,
    wallet_ops: WalletOperations,
    pool: Arc<SqlitePool>,
    merkle_manager: MerkleTreeManager,
    pool_for_wallet_manager: Arc<SqlitePool>, //  暂时使用 pool 创建 WalletManager,  后续考虑更优雅的依赖注入方式
}

impl TransactionManager {
    /// 创建新的交易管理器实例
    pub async fn new(pool: Arc<SqlitePool>) -> Result<Self, LedgerError> {
        let db_ops = TransactionOperations::new(pool.clone());
        let wallet_ops = WalletOperations::new(pool.clone());
        let merkle_manager = MerkleTreeManager::new(pool.clone());

        Ok(Self {
            db_ops,
            wallet_ops,
            pool: pool.clone(),
            merkle_manager,
            pool_for_wallet_manager: pool.clone(), //  暂时使用 pool 创建 WalletManager
        })
    }

    /// 获取交易管理信息
    pub async fn get_transaction_management(&self) -> Result<TransactionManagement, LedgerError> {
        debug!("正在获取交易管理信息...");
        let pending = self.get_pending_transactions().await?;
        let confirmed = self.get_confirmed_transactions().await?;

        Ok(TransactionManagement {
            pending_transactions: pending,
            confirmed_transactions: confirmed,
            block_hash: None,
        })
    }

    /// 获取待处理交易列表
    pub async fn get_pending_transactions(&self) -> Result<Vec<TransactionDetail>, LedgerError> {
        debug!("正在获取待处理交易...");
        self.db_ops.get_pending_transactions().await
    }

    /// 获取已确认交易列表
    pub async fn get_confirmed_transactions(&self) -> Result<Vec<TransactionDetail>, LedgerError> {
        debug!("正在获取已确认交易...");
        let pool = &*self.pool;

        // 定义数据库行结构
        #[derive(sqlx::FromRow)]
        struct TransactionRow {
            hash: String,
            transaction_type: String,
            from_address: String,
            to_address: String,
            transfer_amount: String,
            nonce: i64,
            signature: String,
            timestamp: i64,
            fee: String,
            locked: bool,
            unlocked_time: i64,
            transaction_status: String,
        }

        // 查询已确认的交易
        let rows = sqlx::query_as::<_, TransactionRow>(
            "SELECT * FROM transactions WHERE status = 'confirmed'",
        )
        .fetch_all(pool)
        .await?;

        // 转换为 Transaction 类型
        let transactions = rows
            .into_iter()
            .map(|row| {
                Ok(TransactionDetail {
                    transaction_type: serde_json::from_str(&row.transaction_type)?,
                    from: row.from_address,
                    to: row.to_address,
                    transfer_amount: Amount::from_str(&row.transfer_amount)
                        .map_err(|e| LedgerError::AmountError(e.to_string()))?,
                    nonce: row.nonce as i64,
                    initiator_signature: SignatureWrapper::from_bytes(&hex::decode(row.signature)?)
                        .map_err(|e| LedgerError::SignatureError(e.to_string()))?,
                    timestamp: chrono::DateTime::from_timestamp(row.timestamp, 0)
                        .ok_or_else(|| LedgerError::TimestampError("无效的时间戳".to_string()))?,
                    fee: Amount::from_str(&row.fee)
                        .map_err(|e| LedgerError::AmountError(e.to_string()))?,
                    transaction_hash: to_hash(hex::decode(row.hash)?)
                        .map_err(|e| LedgerError::InvalidData(e.to_string()))?,
                    locked: row.locked,
                    unlocked_time: chrono::DateTime::from_timestamp(row.unlocked_time, 0)
                        .ok_or_else(|| LedgerError::TimestampError("无效的时间戳".to_string()))?,
                    transaction_status: row.transaction_status,
                })
            })
            .collect::<Result<Vec<_>, LedgerError>>()?;

        debug!("加载到 {} 笔已确认交易", transactions.len());
        Ok(transactions)
    }

    /// 清理已确认的交易
    pub async fn clean_confirmed_transactions(&self) -> Result<(), LedgerError> {
        debug!("正在清理已确认交易...");
        self.db_ops.clean_confirmed_transactions().await
    }

    /// 构建交易列表的默克尔树
    pub async fn build_transaction_merkle_tree(
        &mut self,
        transactions: &[TransactionDetail],
    ) -> Result<Hash, LedgerError> {
        debug!(
            "TransactionManager 正在构建交易默克尔树，交易数量: {}",
            transactions.len()
        );
        self.merkle_manager.build_merkle_tree(transactions).await
    }

    /// 验证交易
    pub async fn verify_transaction(
        &self,
        transaction: &TransactionDetail,
    ) -> Result<bool, LedgerError> {
        debug!("正在验证交易: {:?}", transaction);

        // 验证地址
        if !self.verify_addresses(transaction).await? {
            return Ok(false);
        }

        //  验证余额
        let wallet = match self.wallet_ops.get_wallet(&transaction.from).await? {
            Some(wallet) => wallet,
            None => {
                error!("发送方钱包不存在: {}", transaction.from);
                return Ok(false);
            }
        };
        let sender_balance = wallet.balance;

        // 计算总金额 (使用 Amount 实现的 Add trait)
        let total_amount = match transaction.transfer_amount.clone() + transaction.fee.clone() {
            Ok(amount) => amount,
            Err(e) => {
                error!("计算总金额失败: {}", e);
                return Ok(false);
            }
        };

        if sender_balance < total_amount {
            error!(
                "余额不足: 余额={}, 需要={}",
                sender_balance.to_string(),
                total_amount.to_string()
            );
            return Ok(false);
        }

        //  验证nonce
        if !self.verify_nonce(transaction).await? {
            return Ok(false);
        }

        //  签名验证
        debug!("开始签名验证...");
        let transaction_bytes =
            serde_json::to_vec(transaction).map_err(|e| LedgerError::SerializationError(e))?;

        // 直接将 transaction.from (地址) 传递给 verify 方法
        let sender_address = &transaction.from;

        if !transaction
            .initiator_signature
            .verify(&transaction_bytes, sender_address) //  使用 sender_address
            .map_err(|e| LedgerError::SignatureError(e))?
        {
            error!("交易签名验证失败!");
            return Ok(false);
        }
        debug!("签名验证通过!");

        Ok(true) // 暂时只做签名验证，后续添加其他验证
    }

    // 验证地址格式和有效性
    async fn verify_addresses(&self, transaction: &TransactionDetail) -> Result<bool, LedgerError> {
        use crate::wallet::address::WalletAddress;

        // 1. 验证地址是否为空
        if transaction.from.is_empty() || transaction.to.is_empty() {
            error!("地址不能为空");
            return Ok(false);
        }

        // 2. 验证地址格式
        if !WalletAddress::validate_address(&transaction.from)
            || !WalletAddress::validate_address(&transaction.to)
        {
            error!(
                "地址格式无效: from={}, to={}",
                transaction.from, transaction.to
            );
            return Ok(false);
        }

        // 3. 验证发送方地址是否存在
        if let None = self.wallet_ops.get_wallet(&transaction.from).await? {
            error!("发送方地址不存在: {}", transaction.from);
            return Ok(false);
        }

        // 4. 验证是否自己给自己转账
        if transaction.from == transaction.to {
            error!("不能自己给自己转账");
            return Ok(false);
        }

        Ok(true)
    }

    /// 验证交易的nonce
    ///
    /// # 参数
    /// * `transaction` - 待验证的交易
    ///
    /// # 返回值
    /// * `Result<bool, LedgerError>` - 验证结果
    async fn verify_nonce(&self, transaction: &TransactionDetail) -> Result<bool, LedgerError> {
        let sender_address = &transaction.from;

        // 从数据库获取账户当前nonce
        let current_nonce = self.wallet_ops.get_account_nonce(sender_address).await?;

        // nonce必须等于当前nonce + 1
        if transaction.nonce as i64 != current_nonce + 1 {
            error!(
                "无效的nonce值: 期望 {}, 实际 {}",
                current_nonce + 1,
                transaction.nonce
            );
            return Ok(false);
        }

        debug!("nonce验证通过: {}", transaction.nonce);
        Ok(true)
    }

    /// 更新账户nonce
    async fn update_account_nonce(&self, address: &str, new_nonce: i64) -> Result<(), LedgerError> {
        self.wallet_ops
            .safe_update_nonce(address, new_nonce - 1, new_nonce)
            .await
    }

    /// 同步交易管理信息到数据库
    pub async fn sync_to_db(&self, management: &TransactionManagement) -> Result<(), LedgerError> {
        debug!("正在同步交易管理信息到数据库...");

        // 开始事务
        let transaction = self.pool.begin().await?;

        let result = async {
            // 同步待处理交易
            for tx in &management.pending_transactions {
                self.db_ops.insert_transaction(tx.clone()).await?;
            }

            // 同步已确认交易
            for tx in &management.confirmed_transactions {
                self.db_ops
                    .update_transaction_status(&hex::encode(&tx.transaction_hash), "confirmed")
                    .await?;
            }
            Ok::<(), LedgerError>(())
        }
        .await;

        match result {
            Ok(_) => {
                transaction.commit().await?;
                debug!("交易管理信息同步完成");
                Ok(())
            }
            Err(e) => {
                error!("同步失败，正在回滚事务: {}", e);
                transaction.rollback().await?;
                Err(e)
            }
        }
    }
}
