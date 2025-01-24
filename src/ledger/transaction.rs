use crate::types::ledger::TransactionManagement;
use crate::types::transaction::Transaction;
use super::error::LedgerError;
use super::db::operation::TransactionOperations;
use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use log::debug;

pub struct TransactionManager {
    db_ops: TransactionOperations,
    conn: Arc<Mutex<Connection>>,
}

impl TransactionManager {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Result<Self, LedgerError> {
        let db_ops = TransactionOperations::new(conn.clone());
        Ok(Self {
            db_ops,
            conn: conn.clone(),
        })
    }

    pub async fn get_transaction_management(&self) -> Result<TransactionManagement, LedgerError> {
        debug!("正在获取交易管理信息...");
        let pending = self.get_pending_transactions().await?;
        let confirmed = self.db_ops.get_confirmed_transactions()?;
        
        Ok(TransactionManagement {
            pending_transactions: pending,
            confirmed_transactions: confirmed,
            block_hash: None,
        })
    }

    pub async fn get_pending_transactions(&self) -> Result<Vec<Transaction>, LedgerError> {
        debug!("正在获取待处理交易...");
        self.db_ops.get_pending_transactions()
    }

    pub async fn clean_confirmed_transactions(&self) -> Result<(), LedgerError> {
        debug!("正在清理已确认交易...");
        self.db_ops.clean_confirmed_transactions()
    }

    pub async fn sync_to_db(&self, management: &TransactionManagement) -> Result<(), LedgerError> {
        debug!("正在同步交易管理信息到数据库...");
        // 同步待处理交易
        for transaction in &management.pending_transactions {
            self.db_ops.insert_transaction(transaction.clone())?;
        }
        
        // 同步已确认交易
        for transaction in &management.confirmed_transactions {
            self.db_ops.update_transaction_status(
                &transaction.transaction_hash.to_string(), 
                "confirmed"
            )?;
        }
        
        debug!("交易管理信息同步完成");
        Ok(())
    }
}