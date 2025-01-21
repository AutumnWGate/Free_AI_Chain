pub mod error;
pub mod wallet;
pub mod transaction;
pub mod block;
pub mod merkletree;
pub mod db;

use log::{debug, error, info};
use crate::types::ledger::LedgerState;
use crate::types::wallet::Wallet;
use crate::types::block::Block;
use crate::types::transaction::Transaction;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use self::error::LedgerError;
use self::wallet::WalletManager;
use self::transaction::TransactionManager;
use self::block::BlockManager;
use self::db::operation::DatabaseManager;

/// 账本事件类型
#[derive(Debug)]
pub enum LedgerEvent {
    WalletCreated(Wallet),
    TransactionAdded(Transaction),
    BlockCreated(Block),
    StateUpdated,
    Error(LedgerError),
}

/// 账本管理器结构体
pub struct LedgerManager {
    pub state: Arc<Mutex<LedgerState>>,
    pub db_conn: Arc<Mutex<Connection>>,
    wallet_manager: WalletManager,
    transaction_manager: TransactionManager,
    block_manager: BlockManager,
    db_manager: DatabaseManager,
    event_sender: Sender<LedgerEvent>,
    event_receiver: Arc<Mutex<Receiver<LedgerEvent>>>,
}

impl LedgerManager {
    /// 创建新的账本管理器实例
    pub fn new(db_path: &str) -> Result<Self, LedgerError> {
        info!("正在初始化账本管理器...");
        
        // 初始化数据库连接
        let conn = Connection::open(db_path)
            .map_err(|e| {
                error!("数据库连接失败: {}", e);
                LedgerError::DatabaseError(e)
            })?;
        
        let db_conn = Arc::new(Mutex::new(conn));
        
        // 初始化数据库管理器
        let db_manager = DatabaseManager::new(Arc::clone(&db_conn));
        db_manager.initialize_tables()?;
        
        // 初始化各个管理器
        let wallet_manager = WalletManager::new(Arc::clone(&db_conn))?;
        let transaction_manager = TransactionManager::new(Arc::clone(&db_conn))?;
        let block_manager = BlockManager::new(Arc::clone(&db_conn))?;
        let (sender, receiver) = channel();
        let event_receiver = Arc::new(Mutex::new(receiver));
        
        // 初始化账本状态
        let state = Arc::new(Mutex::new(LedgerState {
            wallet_management: wallet_manager.get_wallet_management()?,
            block_management: block_manager.get_block_management()?,
            transaction_management: transaction_manager.get_transaction_management()?,
        }));
        
        info!("账本管理器初始化完成");
        
        Ok(Self {
            state,
            db_conn,
            wallet_manager,
            transaction_manager,
            block_manager,
            db_manager,
            event_sender,
            event_receiver,
        })
    }
    
    /// 获取钱包管理器
    pub fn wallet_manager(&self) -> &WalletManager {
        &self.wallet_manager
    }
    
    /// 获取交易管理器
    pub fn transaction_manager(&self) -> &TransactionManager {
        &self.transaction_manager
    }
    
    /// 获取区块管理器
    pub fn block_manager(&self) -> &BlockManager {
        &self.block_manager
    }

    /// 同步账本状态到数据库
    pub fn sync_to_db(&self) -> Result<(), LedgerError> {
        debug!("正在同步账本状态到数据库...");
        let state = self.state.lock().map_err(|_| {
            error!("获取状态锁失败");
            LedgerError::Unknown("获取状态锁失败".to_string())
        })?;
        
        self.wallet_manager.sync_to_db(&state.wallet_management)?;
        self.transaction_manager.sync_to_db(&state.transaction_management)?;
        self.block_manager.sync_to_db(&state.block_management)?;
        
        debug!("账本状态同步完成");
        Ok(())
    }

    /// 从数据库加载账本状态
    pub fn load_from_db(&self) -> Result<(), LedgerError> {
        debug!("正在从数据库加载账本状态...");
        self.update_state()?;
        debug!("账本状态加载完成");
        Ok(())
    }
    
    /// 更新账本状态
    pub fn update_state(&self) -> Result<(), LedgerError> {
        debug!("正在更新账本状态...");
        let mut state = self.state.lock().map_err(|_| {
            error!("获取状态锁失败");
            LedgerError::Unknown("获取状态锁失败".to_string())
        })?;
        
        *state = LedgerState {
            wallet_management: self.wallet_manager.get_wallet_management()?,
            block_management: self.block_manager.get_block_management()?,
            transaction_management: self.transaction_manager.get_transaction_management()?,
        };
        
        debug!("账本状态更新完成");
        Ok(())
    }

    /// 获取交易池中的交易
    pub fn get_pending_transactions(&self) -> Result<Vec<Transaction>, LedgerError> {
        self.transaction_manager.get_pending_transactions()
    }

    /// 清理已确认的交易
    pub fn clean_confirmed_transactions(&self) -> Result<(), LedgerError> {
        self.transaction_manager.clean_confirmed_transactions()
    }

    /// 获取事件接收器
    pub fn get_event_receiver(&self) -> Arc<Mutex<Receiver<LedgerEvent>>> {
        Arc::clone(&self.event_receiver)
    }

    /// 发送事件
    fn send_event(&self, event: LedgerEvent) {
        if let Err(e) = self.event_sender.send(event) {
            error!("发送事件失败: {:?}", e);
        }
    }
    
    /// 关闭账本管理器
    pub fn shutdown(&self) -> Result<(), LedgerError> {
        info!("正在关闭账本管理器...");
        
        // 同步最终状态到数据库
        if let Err(e) = self.sync_to_db() {
            error!("同步状态到数据库失败: {:?}", e);
        }
        
        // 清理交易池
        if let Err(e) = self.clean_confirmed_transactions() {
            error!("清理已确认交易失败: {:?}", e);
        }
        
        info!("账本管理器关闭完成");
        Ok(())
    }
}

// 为 LedgerManager 实现 Drop trait
impl Drop for LedgerManager {
    fn drop(&mut self) {
        if let Err(e) = self.shutdown() {
            error!("关闭账本管理器时发生错误: {:?}", e);
        }
    }
}