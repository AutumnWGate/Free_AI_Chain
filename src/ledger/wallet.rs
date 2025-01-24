use crate::types::ledger::{WalletManagement, WalletAction};
use crate::types::wallet::Wallet;
use crate::types::amount::Amount;
use super::error::LedgerError;
use super::db::operation::WalletOperations;
use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use log::{debug, error, info};
use crate::types::transaction::Transaction;

pub struct WalletManager {
    db_ops: WalletOperations,
    conn: Arc<Mutex<Connection>>,
}

impl WalletManager {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Result<Self, LedgerError> {
        let db_ops = WalletOperations::new(conn.clone());
        Ok(Self {
            db_ops,
            conn: conn.clone(),
        })
    }

    pub async fn create_wallet(&self) -> Result<Wallet, LedgerError> {
        info!("正在创建钱包...");
        let (wallet, mnemonic) = Wallet::new()
        .map_err(|e| LedgerError::WalletError(e.to_string()))?;
        
        // 使用 db_ops 将钱包信息保存到数据库
        self.db_ops.insert_wallet(&wallet)?;
        
        info!("钱包创建成功，地址: {}, 助记词: {}", wallet.address, mnemonic);
        Ok(wallet)
    }

    pub async fn get_balance(&self, address: &str) -> Result<Amount, LedgerError> {
        debug!("正在查询钱包余额，地址: {}", address);
        
        // 使用 db_ops 从数据库查询钱包余额       
        match self.db_ops.get_wallet(address)? {
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

    pub async fn get_wallet_management(&self) -> Result<WalletManagement, LedgerError> {
        debug!("正在获取钱包管理信息...");
        let conn = self.conn.lock().map_err(|e| {
            error!("获取数据库连接锁失败: {}", e);
            LedgerError::LockError(e.to_string())
        })?;

        let mut stmt = conn.prepare("SELECT address FROM wallets")
            .map_err(|e| {
                error!("准备查询语句失败: {}", e);
                LedgerError::DatabaseError(e)
            })?;

        let addresses: Vec<String> = stmt.query_map([], |row| row.get(0))
            .map_err(|e| {
                error!("查询钱包地址失败: {}", e);
                LedgerError::DatabaseError(e)
            })?
            .collect::<Result<Vec<String>, _>>()
            .map_err(|e| {
                error!("收集钱包地址失败: {}", e);
                LedgerError::DatabaseError(e)
            })?;

        let actions = addresses.into_iter().map(|address| {
            WalletAction::GetBalance { address }
        }).collect();

        debug!("钱包管理信息获取成功");
        let wallet_records = Vec::new();

        debug!("钱包管理信息获取成功");
        Ok(WalletManagement { actions, wallet_records })
    }

    pub async fn get_transaction_history(&self, address: &str) -> Result<Vec<Transaction>, LedgerError> {
        debug!("正在获取钱包交易历史: {}", address);
        self.db_ops.get_transaction_history(address)
    }

    pub async fn sync_to_db(&self, wallet_management: &WalletManagement) -> Result<(), LedgerError> {
        debug!("正在同步钱包管理信息到数据库...");
        for action in &wallet_management.actions {
            match action {
                WalletAction::GetBalance { address } => {
                    let balance = self.get_balance(address).await?;
                    self.db_ops.update_wallet_balance(address, balance)?;
                }
                WalletAction::CreateWallet => {
                    let (wallet, mnemonic) = Wallet::new()
                        .map_err(|e| LedgerError::CreateWalletError(e.to_string()))?;
                    self.db_ops.insert_wallet(&wallet)?;
                    debug!("创建钱包成功，助记词: {}", mnemonic);
                }
                WalletAction::DeleteWallet { address } => {
                    self.db_ops.delete_wallet(address)?;
                }
                WalletAction::GetTransactionHistory { address } => {
                    let transactions = self.get_transaction_history(address).await?;
                    self.db_ops.update_transaction_history(address, &transactions)?;
                }
            }
        }
        debug!("钱包管理信息同步完成");
        Ok(())
    }
}