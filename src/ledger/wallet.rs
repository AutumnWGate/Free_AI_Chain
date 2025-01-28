use crate::types::ledger::{WalletManagement, WalletAction};
use crate::types::wallet::Wallet;
use crate::types::amount::Amount;
use super::error::LedgerError;
use super::db::operation::WalletOperations;
use std::sync::Arc;
use sqlx::SqlitePool;
use log::{debug, error, info};
use crate::types::transaction::Transaction;

pub struct WalletManager {
    db_ops: WalletOperations,
    pool: Arc<SqlitePool>,
}

impl WalletManager {
    /// 创建新的钱包管理器实例
    pub fn new(pool: Arc<SqlitePool>) -> Result<Self, LedgerError> {
        let db_ops = WalletOperations::new(pool.clone());
        Ok(Self {
            db_ops,
            pool,
        })
    }

    /// 创建新钱包
    pub async fn create_wallet(&self) -> Result<Wallet, LedgerError> {
        info!("正在创建钱包...");
        let (wallet, mnemonic) = Wallet::create_wallet()
            .map_err(|e| LedgerError::WalletError(e.to_string()))?;
        
        // 使用 db_ops 将钱包信息保存到数据库
        self.db_ops.insert_wallet(&wallet).await?;
        
        info!("钱包创建成功，地址: {}, 助记词: {}", wallet.address, mnemonic);
        Ok(wallet)
    }

    /// 获取钱包余额
    pub async fn get_balance(&self, address: &str) -> Result<Amount, LedgerError> {
        debug!("正在查询钱包余额，地址: {}", address);
        
        // 使用 db_ops 从数据库查询钱包余额       
        match self.db_ops.get_wallet(address).await? {
            Some(wallet) => {
                debug!("钱包余额: {}", wallet.balance.to_string());
                Ok(wallet.balance)
            },
            None => {
                error!("钱包不存在: {}", address);
                Err(LedgerError::WalletError(format!("钱包不存在: {}", address)))
            }
        }
    }

    pub async fn update_wallet_balance(&self, address: &str) -> Result<(), LedgerError> {
        debug!("更新钱包余额: {}", address);
        let balance = self.get_balance(address).await?;
        self.db_ops.update_wallet_balance(address, balance).await
    }

    /// 获取钱包管理信息
    pub async fn get_wallet_management(&self, addresses: Option<Vec<String>>) -> Result<WalletManagement, LedgerError> {
        debug!("正在获取钱包管理信息...");
        
        // 可选择性地处理指定的钱包地址
        let actions = match addresses {
            Some(addrs) => addrs.into_iter()
                .map(|address| WalletAction::GetBalance { address })
                .collect(),
            None => Vec::new()
        };
    
        Ok(WalletManagement { 
            actions,
            wallet_records: Vec::new()
        })
    }


    /// 获取钱包交易历史
    pub async fn get_transaction_history(&self, address: &str) -> Result<Vec<Transaction>, LedgerError> {
        debug!("正在获取钱包交易历史: {}", address);
        self.db_ops.get_transaction_history(address).await
    }

    /// 同步钱包管理信息到数据库
    pub async fn sync_to_db(&self, wallet_management: &WalletManagement) -> Result<(), LedgerError> {
        debug!("正在同步钱包管理信息到数据库...");
        
        // 开始事务
        let transaction = self.pool.begin().await?;
        
        let result = async {
            for action in &wallet_management.actions {
                match action {
                    WalletAction::GetBalance { address } => {
                        let balance = self.get_balance(address).await?;
                        self.db_ops.update_wallet_balance(address, balance).await?;
                    }
                    WalletAction::CreateWallet => {
                        let (wallet, mnemonic) = Wallet::create_wallet()
                            .map_err(|e| LedgerError::CreateWalletError(e.to_string()))?;
                        self.db_ops.insert_wallet(&wallet).await?;
                        debug!("创建钱包成功，助记词: {}", mnemonic);
                    }
                    WalletAction::DeleteWallet { address } => {
                        self.db_ops.delete_wallet(address).await?;
                    }
                    WalletAction::GetTransactionHistory { address } => {
                        let transactions = self.get_transaction_history(address).await?;
                        self.db_ops.update_transaction_history(address, &transactions).await?;
                    }
                }
            }
            Ok::<(), LedgerError>(())
        }.await;

        match result {
            Ok(_) => {
                transaction.commit().await?;
                debug!("钱包管理信息同步完成");
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