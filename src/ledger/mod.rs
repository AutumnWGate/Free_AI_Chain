pub mod block;
pub mod db;
pub mod error;
pub mod merkletree;
pub mod transaction;
pub mod wallet;

use self::block::BlockManager;
use self::db::operation::DatabaseManager;
use self::error::LedgerError;
use self::merkletree::MerkleTreeManager;
use self::transaction::TransactionManager;
use self::wallet::WalletManager;
use crate::types::block::Block;
use crate::types::ledger::LedgerState;
use crate::types::transaction::Transaction;
use crate::types::wallet::Wallet;
use log::{debug, error, info};
use sqlx::SqlitePool;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

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
    pub pool: Arc<SqlitePool>, // 修改为 SqlitePool
    wallet_manager: WalletManager,
    transaction_manager: TransactionManager,
    block_manager: BlockManager,
    db_manager: DatabaseManager,
    event_sender: Sender<LedgerEvent>,
    event_receiver: Arc<Mutex<Receiver<LedgerEvent>>>,
    merkle_manager: Arc<Mutex<MerkleTreeManager>>,
}

impl LedgerManager {
    /// 创建新的账本管理器实例
    pub async fn new(db_path: &str) -> Result<Self, LedgerError> {
        info!("正在初始化账本管理器...");

        // 初始化数据库连接池
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(5)
            .min_connections(1)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .idle_timeout(std::time::Duration::from_secs(300))
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    // 设置外键约束
                    sqlx::query("PRAGMA foreign_keys = ON")
                        .execute(&mut *conn)
                        .await?;
                    // 设置WAL模式
                    sqlx::query("PRAGMA journal_mode = WAL")
                        .execute(&mut *conn)
                        .await?;
                    Ok(())
                })
            })
            .connect(db_path)
            .await
            .map_err(|e| {
                error!("数据库连接失败: {}", e);
                LedgerError::DatabaseError(e)
            })?;

        let pool = Arc::new(pool);

        // 初始化数据库表
        let db_manager = DatabaseManager::new(pool.clone());
        db_manager.initialize_tables().await?;

        // 初始化各个管理器
        let transaction_manager = TransactionManager::new(pool.clone()).await?;
        let wallet_manager = WalletManager::new(pool.clone())?;

        let merkle_manager = Arc::new(Mutex::new(MerkleTreeManager::new(pool.clone())));
        let block_manager = BlockManager::new(pool.clone(), merkle_manager.clone())?;

 

        let (sender, receiver) = channel();
        let event_receiver = Arc::new(Mutex::new(receiver));

        // 初始化账本状态
        let state = Arc::new(Mutex::new(LedgerState {
            wallet_management: wallet_manager.get_wallet_management(None).await?,
            block_management: block_manager.get_block_management().await?,
            transaction_management: transaction_manager.get_transaction_management().await?,
        }));

        info!("账本管理器初始化完成");

        Ok(Self {
            state,
            pool, // 使用 pool 替代 db_conn
            wallet_manager,
            transaction_manager,
            block_manager,
            db_manager,
            event_sender: sender,
            event_receiver,
            merkle_manager,
        })
    }

    /// 检查数据库连接健康状态
    pub async fn check_database_health(&self) -> Result<(), LedgerError> {
        debug!("正在检查数据库连接健康状态...");
        sqlx::query("SELECT 1")
            .execute(&*self.pool)
            .await
            .map_err(|e| {
                error!("数据库健康检查失败: {}", e);
                LedgerError::DatabaseError(e)
            })?;
        debug!("数据库连接正常");
        Ok(())
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

    /// 数据库迁移支持
    pub async fn run_migrations(&self) -> Result<(), LedgerError> {
        debug!("正在运行数据库迁移...");
        // 数据库迁移支持TODO
        // sqlx::migrate!("./migrations")
        //     .run(&*self.pool)
        //     .await
        //     .map_err(|e| {
        //         error!("数据库迁移失败: {}", e);
        //         LedgerError::DatabaseError(e)
        //     })?;
        debug!("数据库迁移完成");
        Ok(())
    }

    /// 同步账本状态到数据库
    pub async fn sync_to_db(&self) -> Result<(), LedgerError> {
        debug!("正在同步账本状态到数据库...");

        let transaction = self.pool.begin().await?;

        let result = async {
            let state = self.state.lock()?;

            self.wallet_manager
                .sync_to_db(&state.wallet_management)
                .await?;
            self.transaction_manager
                .sync_to_db(&state.transaction_management)
                .await?;
            self.block_manager
                .sync_to_db(&state.block_management)
                .await?;

            Ok::<(), LedgerError>(())
        }
        .await;

        match result {
            Ok(_) => {
                transaction.commit().await?;
                debug!("账本状态同步完成");
                Ok(())
            }
            Err(e) => {
                error!("同步失败，正在回滚事务: {}", e);
                transaction.rollback().await?;
                Err(e)
            }
        }
    }

    /// 从数据库加载账本状态
    pub async fn load_from_db(&self) -> Result<(), LedgerError> {
        debug!("正在从数据库加载账本状态...");
        self.update_state().await?;
        debug!("账本状态加载完成");
        Ok(())
    }

    pub async fn update_state(&self) -> Result<(), LedgerError> {
        debug!("正在更新账本状态...");
        let mut state = self.state.lock().map_err(|_| {
            error!("获取状态锁失败");
            LedgerError::Unknown("获取状态锁失败".to_string())
        })?;

        *state = LedgerState {
            wallet_management: self.wallet_manager.get_wallet_management(None).await?,
            block_management: self.block_manager.get_block_management().await?,
            transaction_management: self
                .transaction_manager
                .get_transaction_management()
                .await?,
        };

        debug!("账本状态更新完成");
        Ok(())
    }

    /// 获取交易池中的交易
    pub async fn get_pending_transactions(&self) -> Result<Vec<Transaction>, LedgerError> {
        self.transaction_manager.get_pending_transactions().await
    }

    /// 清理已确认的交易
    pub async fn clean_confirmed_transactions(&self) -> Result<(), LedgerError> {
        self.transaction_manager
            .clean_confirmed_transactions()
            .await
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
    pub async fn shutdown(&self) -> Result<(), LedgerError> {
        debug!("正在关闭账本管理器...");

        // 同步最终状态
        self.sync_to_db().await?;

        // 等待所有连接释放
        self.pool.close().await;

        debug!("账本管理器已关闭");
        Ok(())
    }
}

// 为 LedgerManager 实现 Drop trait
impl Drop for LedgerManager {
    fn drop(&mut self) {
        // 由于 Drop 不能是异步的，我们只能在这里记录错误
        error!("警告：账本管理器被销毁时无法执行异步清理操作");
    }
}
