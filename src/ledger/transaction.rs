use super::db::operation::TransactionOperations;
use super::error::LedgerError;
use super::merkletree::MerkleTreeManager;
use crate::crypto::hash::to_hash;
use crate::crypto::hash::Hash;
use crate::crypto::signature::SignatureWrapper;
use crate::types::amount::Amount;
use crate::types::ledger::TransactionManagement;
use crate::types::transaction::Transaction;
use log::{debug, error};
use sqlx::SqlitePool;
use std::sync::Arc;

/// 交易管理器
pub struct TransactionManager {
    db_ops: TransactionOperations,
    pool: Arc<SqlitePool>,
    merkle_manager: MerkleTreeManager,
    pool_for_wallet_manager: Arc<SqlitePool>, //  暂时使用 pool 创建 WalletManager,  后续考虑更优雅的依赖注入方式
}

impl TransactionManager {
    /// 创建新的交易管理器实例
    pub fn new(pool: Arc<SqlitePool>) -> Result<Self, LedgerError> {
        let db_ops = TransactionOperations::new(pool.clone());
        let merkle_manager = MerkleTreeManager::new(pool.clone());
        Ok(Self {
            db_ops,
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
    pub async fn get_pending_transactions(&self) -> Result<Vec<Transaction>, LedgerError> {
        debug!("正在获取待处理交易...");
        self.db_ops.get_pending_transactions().await
    }

    /// 获取已确认交易列表
    pub async fn get_confirmed_transactions(&self) -> Result<Vec<Transaction>, LedgerError> {
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
                Ok(Transaction {
                    transaction_type: serde_json::from_str(&row.transaction_type)?,
                    from: row.from_address,
                    to: row.to_address,
                    transfer_amount: Amount::from_str(&row.transfer_amount)
                        .map_err(|e| LedgerError::AmountError(e.to_string()))?,
                    nonce: row.nonce as u64,
                    signature: SignatureWrapper::from_bytes(&hex::decode(row.signature)?)
                        .map_err(|e| LedgerError::SignatureError(e.to_string()))?,
                    timestamp: chrono::DateTime::from_timestamp(row.timestamp, 0)
                        .ok_or_else(|| LedgerError::TimestampError("无效的时间戳".to_string()))?,
                    fee: Amount::from_str(&row.fee)
                        .map_err(|e| LedgerError::AmountError(e.to_string()))?,
                    transaction_hash: to_hash(hex::decode(row.hash)?)
                        .map_err(|e| LedgerError::InvalidData(e.to_string()))?,
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
        transactions: &[Transaction],
    ) -> Result<Hash, LedgerError> {
        debug!(
            "TransactionManager 正在构建交易默克尔树，交易数量: {}",
            transactions.len()
        );
        self.merkle_manager.build_merkle_tree(transactions).await
    }

    /// 验证交易
    pub async fn verify_transaction(&self, transaction: &Transaction) -> Result<bool, LedgerError> {
        debug!("正在验证交易: {:?}", transaction);
        // 1. 签名验证
        debug!("开始签名验证...");
        let transaction_bytes =
            serde_json::to_vec(transaction).map_err(|e| LedgerError::SerializationError(e))?;

        // 直接将 transaction.from (地址) 传递给 verify 方法
        let sender_address = &transaction.from;

        if !transaction
            .signature
            .verify(&transaction_bytes, sender_address) //  使用 sender_address
            .map_err(|e| LedgerError::SignatureError(e))?
        {
            error!("交易签名验证失败!");
            return Ok(false);
        }
        debug!("签名验证通过!");

        Ok(true) // 暂时只做签名验证，后续添加其他验证
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
