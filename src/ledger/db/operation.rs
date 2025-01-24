use sqlx::{Connection, Pool};
use sqlx::sqlite::{SqlitePool, Sqlite};
use sqlx::Type;
use std::sync::{Arc, Mutex};
use log::{debug, info, error};
use super::super::error::LedgerError;
use crate::types::wallet::Wallet;
use crate::types::transaction::Transaction;
use crate::types::amount::Amount;
use crate::types::block::{Block, BlockHeader};
use crate::crypto::signature::SignatureWrapper;
use crate::crypto::hash::to_hash;

/// 数据库管理器
pub struct DatabaseManager {
    pool: Arc<SqlitePool>,
}

impl DatabaseManager {
    /// 创建新的数据库管理器实例
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    /// 初始化数据库表
    pub async fn initialize_tables(&self) -> Result<(), LedgerError> {
        debug!("正在初始化数据库表...");
        let pool = &*self.pool;

        // 创建钱包表
        debug!("正在创建钱包表...");
        match sqlx::query(
            "CREATE TABLE IF NOT EXISTS wallets (
                address TEXT PRIMARY KEY,
                balance TEXT NOT NULL,
                nonce INTEGER NOT NULL
            )"
        )
        .execute(pool)
        .await {
            Ok(_) => debug!("钱包表创建成功"),
            Err(e) => {
                error!("创建钱包表失败: {}", e);
                return Err(LedgerError::DatabaseError(e.to_string()));
            }
        };

        // 创建交易表
        debug!("正在创建交易表...");
        match sqlx::query(
            "CREATE TABLE IF NOT EXISTS transactions (
                hash TEXT PRIMARY KEY,
                transaction_type TEXT NOT NULL,
                from_address TEXT NOT NULL,
                to_address TEXT NOT NULL,
                transfer_amount TEXT NOT NULL,
                nonce INTEGER NOT NULL,
                signature TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                fee TEXT NOT NULL,
                block_hash TEXT,
                status TEXT NOT NULL
            )"
        )
        .execute(pool)
        .await {
            Ok(_) => debug!("交易表创建成功"),
            Err(e) => {
                error!("创建交易表失败: {}", e);
                return Err(LedgerError::DatabaseError(e.to_string()));
            }
        };

        // 创建区块表
        debug!("正在创建区块表...");
        match sqlx::query(
            "CREATE TABLE IF NOT EXISTS blocks (
                block_hash TEXT PRIMARY KEY,
                parent_hash TEXT NOT NULL,
                height INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                merkle_root TEXT NOT NULL,
                validator TEXT NOT NULL,
                signature TEXT NOT NULL
            )"
        )
        .execute(pool)
        .await {
            Ok(_) => debug!("区块表创建成功"),
            Err(e) => {
                error!("创建区块表失败: {}", e);
                return Err(LedgerError::DatabaseError(e.to_string()));
            }
        };

        // 创建钱包交易历史表
        debug!("正在创建钱包交易历史表...");
        match sqlx::query(
            "CREATE TABLE IF NOT EXISTS wallet_transaction_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                wallet_address TEXT NOT NULL,
                transaction_hash TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                FOREIGN KEY (wallet_address) REFERENCES wallets(address),
                FOREIGN KEY (transaction_hash) REFERENCES transactions(hash),
                UNIQUE(wallet_address, transaction_hash)
            )"
        )
        .execute(pool)
        .await {
            Ok(_) => debug!("钱包交易历史表创建成功"),
            Err(e) => {
                error!("创建钱包交易历史表失败: {}", e);
                return Err(LedgerError::DatabaseError(e.to_string()));
            }
        };   

        // 创建索引
        debug!("正在创建索引...");
        for (index_name, query) in [
            ("交易发送方索引", "CREATE INDEX IF NOT EXISTS idx_transactions_from ON transactions(from_address)"),
            ("交易接收方索引", "CREATE INDEX IF NOT EXISTS idx_transactions_to ON transactions(to_address)"),
            ("交易区块索引", "CREATE INDEX IF NOT EXISTS idx_transactions_block ON transactions(block_hash)"),
            ("区块高度索引", "CREATE INDEX IF NOT EXISTS idx_blocks_height ON blocks(height)"),
            ("区块父哈希索引", "CREATE INDEX IF NOT EXISTS idx_blocks_parent ON blocks(parent_hash)"),
            ("钱包历史索引", "CREATE INDEX IF NOT EXISTS idx_wallet_history_address ON wallet_transaction_history(wallet_address)")
        ] {
            debug!("正在创建{}...", index_name);
            match sqlx::query(query)
                .execute(pool)
                .await {
                    Ok(_) => debug!("{}创建成功", index_name),
                    Err(e) => {
                        error!("创建{}失败: {}", index_name, e);
                        return Err(LedgerError::DatabaseError(e.to_string()));
                    }
                };
        }

        info!("数据库表初始化完成");
        Ok(())
    }
}

/// 钱包数据库操作
pub struct WalletOperations {
    pool: Arc<SqlitePool>,
}

/// 交易数据库操作
pub struct TransactionOperations {
    pool: Arc<SqlitePool>,
}



impl WalletOperations {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    pub async fn insert_wallet(&self, wallet: &Wallet) -> Result<(), LedgerError> {
        debug!("正在插入钱包记录: {}", wallet.address);
        let pool = &*self.pool;
        
        match sqlx::query(
            "INSERT INTO wallets (address, balance, nonce) VALUES ($1, $2, $3)"
        )
        .bind(&wallet.address)
        .bind(&wallet.balance.to_string())
        .bind(wallet.nonce as i64)  // 将 u64 转换为 i64
        .execute(pool)
        .await {
            Ok(_) => {
                debug!("钱包记录插入成功");
                Ok(())
            },
            Err(e) => {
                error!("插入钱包记录失败: {}", e);
                Err(LedgerError::DatabaseError(e.to_string()))
            }
        }
    }

    pub async fn get_wallet(&self, address: &str) -> Result<Option<Wallet>, LedgerError> {
        debug!("正在查询钱包: {}", address);
        let pool = &*self.pool;
    
        #[derive(sqlx::FromRow)]
        struct WalletRow {
            address: String,
            balance: String,
            nonce: i64,
        }
    
        match sqlx::query_as::<_, WalletRow>(
            "SELECT address, balance, nonce FROM wallets WHERE address = $1"
        )
        .bind(address)
        .fetch_optional(pool)
        .await {
            Ok(Some(row)) => {
                debug!("钱包查询成功");
                Ok(Some(Wallet {
                    address: row.address,
                    balance: Amount::from_str(&row.balance)
                        .map_err(|e| LedgerError::DatabaseError(e.to_string()))?,
                    nonce: row.nonce as u64,
                }))
            },
            Ok(None) => {
                debug!("钱包不存在");
                Ok(None)
            },
            Err(e) => {
                error!("查询钱包失败: {}", e);
                Err(LedgerError::DatabaseError(e.to_string()))
            }
        }
    }
    
    /// 更新钱包余额,转账成功后的余额
    pub async fn update_wallet_balance(&self, address: &str, amount: Amount) -> Result<(), LedgerError> {
        debug!("正在更新钱包余额: {} -> {}", address, amount.to_string());
        let pool = &*self.pool;

        match sqlx::query(
            "UPDATE wallets SET balance = $1 WHERE address = $2"
        )
        .bind(&amount.to_string())
        .bind(address)
        .execute(pool)
        .await {
            Ok(_) => {
                debug!("钱包余额更新成功");
                Ok(())
            },
            Err(e) => {
                error!("更新钱包余额失败: {}", e);
                Err(LedgerError::DatabaseError(e.to_string()))
            }
        }
    }

    pub async fn delete_wallet(&self, address: &str) -> Result<(), LedgerError> {
        debug!("正在删除钱包: {}", address);
        let pool = &*self.pool;

        match sqlx::query(
            "DELETE FROM wallets WHERE address = $1"
        )
        .bind(address)
        .execute(pool)
        .await {
            Ok(_) => {
                debug!("钱包删除成功");
                Ok(())
            },
            Err(e) => {
                error!("删除钱包失败: {}", e);
                Err(LedgerError::DatabaseError(e.to_string()))
            }
        }
    }

    pub fn update_transaction_history(&self, address: &str, transactions: &[Transaction]) -> Result<(), LedgerError> {
        debug!("正在更新钱包交易历史: {}", address);
        let mut conn = self.conn.lock()?;
        
        // 开始事务，使用更具描述性的变量名
        let db_transaction = conn.transaction().map_err(LedgerError::DatabaseError)?;
        
        for transaction_record in transactions {
            let hash = hex::encode(&transaction_record.transaction_hash);
            // 插入或忽略重复记录
            db_transaction.execute(
                "INSERT OR IGNORE INTO wallet_transaction_history 
                (wallet_address, transaction_hash, timestamp) 
                VALUES (?1, ?2, ?3)",
                params![
                    address,
                    hash,
                    transaction_record.timestamp.timestamp()
                ],
            ).map_err(LedgerError::DatabaseError)?;
        }
        
        // 提交事务
        db_transaction.commit().map_err(LedgerError::DatabaseError)?;
        
        debug!("交易历史更新成功");
        Ok(())
    }

    pub fn get_transaction_history(&self, address: &str) -> Result<Vec<Transaction>, LedgerError> {
        debug!("正在获取钱包交易历史: {}", address);
        let conn = self.conn.lock()?;
        
        let mut stmt = conn.prepare(
            "SELECT t.* FROM transactions t
            INNER JOIN wallet_transaction_history wth ON t.hash = wth.transaction_hash
            WHERE wth.wallet_address = ?1
            ORDER BY wth.timestamp DESC"
        ).map_err(LedgerError::DatabaseError)?;
        
        let transactions = stmt.query_map([address], |row| {
            Ok(Transaction {
                transaction_type: serde_json::from_str(&row.get::<_, String>(1)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        1,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
                from: row.get(2)?,
                to: row.get(3)?,
                transfer_amount: Amount::from_str(&row.get::<_, String>(4)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
                nonce: row.get::<_, String>(5)?
                    .parse::<u64>()  // 或其他适当的数字类型
                    .map_err(|e: std::num::ParseIntError| rusqlite::Error::FromSqlConversionFailure(
                        5,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
                signature: SignatureWrapper::from_bytes(&hex::decode(row.get::<_, String>(6)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
                timestamp: chrono::DateTime::from_timestamp(
                    row.get::<_, String>(7)?
                        .parse::<i64>()
                        .map_err(|e: std::num::ParseIntError| rusqlite::Error::FromSqlConversionFailure(
                            7,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                        ))?,
                    0
                ).ok_or_else(|| rusqlite::Error::FromSqlConversionFailure(
                    7,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid timestamp"))
                ))?,
                fee: Amount::from_str(&row.get::<_, String>(8)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        8,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
                transaction_hash: to_hash(hex::decode(row.get::<_, String>(0)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
            })
        }).map_err(LedgerError::DatabaseError)?;
    
        let result = transactions.collect::<Result<Vec<_>, _>>()
            .map_err(LedgerError::DatabaseError)?;
        
        debug!("获取到 {} 笔交易历史", result.len());
        Ok(result)
    }

}

impl TransactionOperations {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn insert_transaction(&self, transaction: Transaction) -> Result<(), LedgerError> {
        debug!("正在插入交易记录: {:?}", transaction.transaction_hash);
        let conn = self.conn.lock()?;
        
        conn.execute(
            "INSERT INTO transactions (
                hash, transaction_type, from_address, to_address, transfer_amount, 
                nonce, signature, timestamp, fee, block_hash, status
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                &hex::encode(&transaction.transaction_hash),
                &format!("{:?}", transaction.transaction_type),
                &transaction.from,
                &transaction.to,
                &transaction.transfer_amount.to_string(),
                &transaction.nonce,
                &hex::encode(&transaction.signature),
                &transaction.timestamp.timestamp(),
                &transaction.fee,
                "",  // 初始区块哈希为空
                "pending"         // 初始状态为待处理
            ],
        ).map_err(LedgerError::DatabaseError)?;
        
        debug!("交易记录插入成功");
        Ok(())
    }

    pub fn get_pending_transactions(&self) -> Result<Vec<Transaction>, LedgerError> {
        debug!("正在获取待处理交易");
        let conn = self.conn.lock()?;
        
        let mut stmt = conn.prepare(
            "SELECT * FROM transactions WHERE status = 'pending' ORDER BY fee DESC"
        ).map_err(LedgerError::DatabaseError)?;
        
        let transactions = stmt.query_map([], |row| {
            Ok(Transaction {
                transaction_type: serde_json::from_str(&row.get::<_, String>(1)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        1,
                        rusqlite::types::Type::Text,
                        Box::new(e)
                    ))?,
                from: row.get(2)?,
                to: row.get(3)?,
                transfer_amount: Amount::from_str(&row.get::<_, String>(4)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
                nonce: row.get::<_, String>(5)?
                    .parse()
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        5,
                        rusqlite::types::Type::Text,
                        Box::new(e)
                    ))?,
                signature: SignatureWrapper::from_bytes(&hex::decode(row.get::<_, String>(6)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Text,
                        Box::new(e)
                    ))?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
                timestamp: chrono::DateTime::from_timestamp(
                    row.get::<_, String>(7)?
                        .parse()
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            7,
                            rusqlite::types::Type::Text,
                            Box::new(e)
                        ))?,
                    0
                ).ok_or_else(|| rusqlite::Error::FromSqlConversionFailure(
                    7,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid timestamp"
                    ))
                ))?,
                fee: Amount::from_str(&row.get::<_, String>(8)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        8,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
                transaction_hash: to_hash(hex::decode(row.get::<_, String>(0)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(e)
                    ))?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
            })
        }).map_err(LedgerError::DatabaseError)?;

        let result = transactions.collect::<Result<Vec<_>, _>>()
            .map_err(LedgerError::DatabaseError)?;
        
        debug!("获取到 {} 笔待处理交易", result.len());
        Ok(result)
    }

    pub fn update_transaction_status(&self, hash: &str, status: &str) -> Result<(), LedgerError> {
        debug!("正在更新交易状态: {} -> {}", hash, status);
        let conn = self.conn.lock()?;
        
        conn.execute(
            "UPDATE transactions SET status = ?1 WHERE hash = ?2",
            [status, hash],
        ).map_err(LedgerError::DatabaseError)?;
        
        debug!("交易状态更新成功");
        Ok(())
    }

    /// 获取已确认的交易列表
    /// 
    /// # 返回值
    /// - `Result<Vec<Transaction>, LedgerError>`: 成功返回已确认的交易列表，失败返回错误
    pub fn get_confirmed_transactions(&self) -> Result<Vec<Transaction>, LedgerError> {
        debug!("正在获取已确认交易");
        let conn = self.conn.lock()?;
        
        let mut stmt = conn.prepare(
            "SELECT * FROM transactions WHERE status = 'confirmed' ORDER BY timestamp DESC"
        ).map_err(LedgerError::DatabaseError)?;
        
        let transactions = stmt.query_map([], |row| {
            Ok(Transaction {
                transaction_type: serde_json::from_str(&row.get::<_, String>(1)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        1,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
                from: row.get(2)?,
                to: row.get(3)?,
                transfer_amount: Amount::from_str(&row.get::<_, String>(4)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
                nonce: row.get::<_, String>(5)?
                    .parse::<u64>()  // 或其他适当的数字类型
                    .map_err(|e: std::num::ParseIntError| rusqlite::Error::FromSqlConversionFailure(
                        5,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
                signature: SignatureWrapper::from_bytes(&hex::decode(row.get::<_, String>(6)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
                timestamp: chrono::DateTime::from_timestamp(
                    row.get::<_, String>(7)?
                        .parse::<i64>()
                        .map_err(|e: std::num::ParseIntError| rusqlite::Error::FromSqlConversionFailure(
                            7,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                        ))?,
                    0
                ).ok_or_else(|| rusqlite::Error::FromSqlConversionFailure(
                    7,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid timestamp"))
                ))?,
                fee: Amount::from_str(&row.get::<_, String>(8)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        8,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
                transaction_hash: to_hash(hex::decode(row.get::<_, String>(0)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
            })
        }).map_err(LedgerError::DatabaseError)?;
    
        let result = transactions.collect::<Result<Vec<_>, _>>()
            .map_err(LedgerError::DatabaseError)?;
        
        debug!("获取到 {} 笔已确认交易", result.len());
        Ok(result)
    }

    /// 清理已确认的交易
    /// 
    /// # 返回值
    /// - `Result<(), LedgerError>`: 成功返回 Ok(()), 失败返回错误
    pub fn clean_confirmed_transactions(&self) -> Result<(), LedgerError> {
        debug!("正在清理已确认交易");
        let conn = self.conn.lock()?;
        
        // 获取一个月前的时间戳
        let one_month_ago = chrono::Utc::now()
            .checked_sub_days(chrono::Days::new(30))
            .ok_or_else(|| LedgerError::InvalidData("Failed to calculate date one month ago".to_string()))?
            .timestamp();
        
        conn.execute(
            "DELETE FROM transactions 
            WHERE status = 'confirmed' 
            AND CAST(timestamp AS INTEGER) < ?1",
            [one_month_ago.to_string()],
        ).map_err(LedgerError::DatabaseError)?;
        
        debug!("已确认交易清理完成");
        Ok(())
    }
}

/// 区块数据库操作
pub struct BlockOperations {
    conn: Arc<Mutex<Connection>>,
}

impl BlockOperations {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    // 辅助方法：加载区块的交易记录
    fn load_block_transactions(&self, block_hash: &str) -> Result<Vec<Transaction>, rusqlite::Error> {
        let conn = self.conn.lock().map_err(|_| rusqlite::Error::InvalidQuery)?;
        
        let mut stmt = conn.prepare(
            "SELECT * FROM transactions WHERE block_hash = ?1"
        )?;
        
        let transactions = stmt.query_map([block_hash], |row| {
            Ok(Transaction {
                transaction_type: serde_json::from_str(&row.get::<_, String>(1)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        1,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
                from: row.get(2)?,
                to: row.get(3)?,
                transfer_amount: Amount::from_str(&row.get::<_, String>(4)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
                nonce: row.get::<_, String>(5)?
                    .parse::<u64>()
                    .map_err(|e: std::num::ParseIntError| rusqlite::Error::FromSqlConversionFailure(
                        5,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
                signature: SignatureWrapper::from_bytes(&hex::decode(row.get::<_, String>(6)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
                timestamp: chrono::DateTime::from_timestamp(
                    row.get::<_, String>(7)?
                        .parse::<i64>()
                        .map_err(|e: std::num::ParseIntError| rusqlite::Error::FromSqlConversionFailure(
                            7,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                        ))?,
                    0
                ).ok_or_else(|| rusqlite::Error::FromSqlConversionFailure(
                    7,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid timestamp"))
                ))?,
                fee: Amount::from_str(&row.get::<_, String>(8)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        8,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
                transaction_hash: to_hash(hex::decode(row.get::<_, String>(0)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
            })
        })?;
    
        transactions.collect()
    }

    pub fn insert_block(&self, block: &Block) -> Result<(), LedgerError> {
        debug!("正在插入区块: {:?}", hex::encode(&block.header.block_hash));
        let mut conn = self.conn.lock()?;

        // 开始事务
        let transaction = conn.transaction().map_err(LedgerError::DatabaseError)?;

        // 1. 遍历所有交易，检查并更新状态
        for block_transaction in &block.transactions {
            let hash = hex::encode(&block_transaction.transaction_hash);
            let status: Option<String> = transaction.query_row(
                "SELECT status FROM transactions WHERE hash = ?1",
                [&hash],
                |row| row.get(0)
            ).optional().map_err(LedgerError::DatabaseError)?;

            match status {
                None => {
                    // 交易不存在，插入交易记录，状态为 confirmed
                    transaction.execute(
                        "INSERT INTO transactions (
                            hash, transaction_type, from_address, to_address, transfer_amount,
                            nonce, signature, timestamp, fee, block_hash, status
                        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'confirmed')",
                        params![
                            &hash,
                            &format!("{:?}", block_transaction.transaction_type),
                            &block_transaction.from,
                            &block_transaction.to,
                            &block_transaction.transfer_amount.to_string(),
                            &block_transaction.nonce,
                            &hex::encode(&block_transaction.signature),
                            &block_transaction.timestamp.timestamp(),
                            &block_transaction.fee.to_string(),
                            &hex::encode(&block.header.block_hash)
                        ],
                    ).map_err(LedgerError::DatabaseError)?;
                },
                Some(current_status) => {
                    if current_status == "pending" {
                        // 交易存在且状态为 pending，更新状态为 confirmed，并关联区块哈希
                        transaction.execute(
                            "UPDATE transactions SET status = 'confirmed', block_hash = ?1 WHERE hash = ?2",
                            [&hex::encode(&block.header.block_hash), &hash],
                        ).map_err(LedgerError::DatabaseError)?;
                    } else if current_status == "confirmed" {
                        // 交易已确认，返回错误
                        return Err(LedgerError::TransactionAlreadyConfirmed(hash));
                    }
                }
            }
        }

        // 2. 插入区块头
        transaction.execute(
            "INSERT INTO blocks (
                block_hash, parent_hash, height, timestamp,
                merkle_root, validator, signature
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            [
                &hex::encode(&block.header.block_hash),
                &hex::encode(&block.header.parent_hash),
                &block.header.height.to_string(),
                &block.header.timestamp.timestamp().to_string(),
                &hex::encode(&block.header.merkle_root),
                &block.header.validator,
                &hex::encode(&block.header.signature),
            ],
        ).map_err(LedgerError::DatabaseError)?;

        // 3. 提交事务
        transaction.commit().map_err(LedgerError::DatabaseError)?;

        debug!("区块及其交易记录插入成功");
        Ok(())
    }

    pub fn get_latest_block(&self) -> Result<Option<Block>, LedgerError> {
        debug!("正在获取最新区块");
        let conn = self.conn.lock()?;
        
        let mut stmt = conn.prepare(
            "SELECT * FROM blocks ORDER BY height DESC LIMIT 1"
        ).map_err(LedgerError::DatabaseError)?;
        
        let block = stmt.query_row([], |row| {
            let header = BlockHeader {
                block_hash: to_hash(hex::decode(row.get::<_, String>(0)?)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                ))?)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                ))?,
                parent_hash: to_hash(hex::decode(row.get::<_, String>(1)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        1,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        1,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
                height: row.get(2)?,
                timestamp: chrono::DateTime::from_timestamp(
                    row.get::<_, String>(3)?
                        .parse::<i64>()
                        .map_err(|e: std::num::ParseIntError| rusqlite::Error::FromSqlConversionFailure(
                            3,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                        ))?,
                    0
                ).ok_or_else(|| rusqlite::Error::FromSqlConversionFailure(
                    3,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid timestamp"))
                ))?,
                merkle_root: to_hash(hex::decode(row.get::<_, String>(4)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
                validator: row.get(5)?,
                signature: SignatureWrapper::from_bytes(&hex::decode(row.get::<_, String>(6)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
                block_number: row.get(2)?,
                previous_block_hash: to_hash(hex::decode(row.get::<_, String>(1)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        1,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        1,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
            };
            
            let block_hash = hex::encode(&header.block_hash);
            let transactions = self.load_block_transactions(&block_hash)?;
            
            Ok(Block {
                header,
                transactions,
            })
        }).optional().map_err(LedgerError::DatabaseError)?;
        
        debug!("最新区块获取成功");
        Ok(block)
    }

    pub fn get_block_by_hash(&self, hash: &str) -> Result<Option<Block>, LedgerError> {
        debug!("正在通过哈希获取区块: {}", hash);
        let conn = self.conn.lock()?;
        
        let mut stmt = conn.prepare(
            "SELECT * FROM blocks WHERE block_hash = ?1"
        ).map_err(LedgerError::DatabaseError)?;
        
        let block = stmt.query_row([hash], |row| {
            let header = BlockHeader {
                block_hash: to_hash(hex::decode(row.get::<_, String>(0)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
                parent_hash: to_hash(hex::decode(row.get::<_, String>(1)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        1,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        1,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
                height: row.get(2)?,
                timestamp: chrono::DateTime::from_timestamp(
                    row.get::<_, String>(3)?
                        .parse::<i64>()
                        .map_err(|e: std::num::ParseIntError| rusqlite::Error::FromSqlConversionFailure(
                            3,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                        ))?,
                    0
                ).ok_or_else(|| rusqlite::Error::FromSqlConversionFailure(
                    3,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid timestamp"))
                ))?,
                merkle_root: to_hash(hex::decode(row.get::<_, String>(4)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
                validator: row.get(5)?,
                signature: SignatureWrapper::from_bytes(&hex::decode(row.get::<_, String>(6)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
                block_number: row.get(2)?,
                previous_block_hash: to_hash(hex::decode(row.get::<_, String>(1)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        1,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        1,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
            };
            
            let transactions = self.load_block_transactions(hash)?;
            
            Ok(Block {
                header,
                transactions,
            })
        }).optional().map_err(LedgerError::DatabaseError)?;
        
        debug!("区块获取成功");
        Ok(block)
    }

    // 获取区块高度
    pub fn get_block_height(&self) -> Result<u64, LedgerError> {
        let conn = self.conn.lock()?;
        let height: Option<u64> = conn.query_row(
            "SELECT MAX(height) FROM blocks",
            [],
            |row| row.get(0)
        ).optional().map_err(LedgerError::DatabaseError)?;
        
        Ok(height.unwrap_or(0))
    }

    pub fn get_block_by_height(&self, height: u64) -> Result<Option<Block>, LedgerError> {
        debug!("正在通过高度获取区块: {}", height);
        let conn = self.conn.lock()?;
        
        let mut stmt = conn.prepare(
            "SELECT * FROM blocks WHERE height = ?1"
        ).map_err(LedgerError::DatabaseError)?;
        
        let block = stmt.query_row([height.to_string()], |row| {
            let header = BlockHeader {
                block_hash: to_hash(hex::decode(row.get::<_, String>(0)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
                parent_hash: to_hash(hex::decode(row.get::<_, String>(1)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        1,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        1,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
                height: row.get(2)?,
                timestamp: chrono::DateTime::from_timestamp(
                    row.get::<_, String>(3)?
                        .parse::<i64>()
                        .map_err(|e: std::num::ParseIntError| rusqlite::Error::FromSqlConversionFailure(
                            3,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                        ))?,
                    0
                ).ok_or_else(|| rusqlite::Error::FromSqlConversionFailure(
                    3,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid timestamp"))
                ))?,
                merkle_root: to_hash(hex::decode(row.get::<_, String>(4)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
                validator: row.get(5)?,
                signature: SignatureWrapper::from_bytes(&hex::decode(row.get::<_, String>(6)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
                block_number: row.get(2)?,
                previous_block_hash: to_hash(hex::decode(row.get::<_, String>(1)?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        1,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        1,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                    ))?,
            };
            
            let block_hash = hex::encode(&header.block_hash);
            let transactions = self.load_block_transactions(&block_hash)?;
            
            Ok(Block {
                header,
                transactions,
            })
        }).optional().map_err(LedgerError::DatabaseError)?;
        
        debug!("区块获取成功");
        Ok(block)
    }

    /// 获取指定范围内的区块
    /// 
    /// # 参数
    /// * `start_height` - 起始高度（包含）
    /// * `end_height` - 结束高度（包含）
    /// 
    /// # 返回值
    /// - `Result<Vec<Block>, LedgerError>`: 成功返回区块列表，失败返回错误
    pub fn get_blocks_by_range(&self, start_height: u64, end_height: u64) -> Result<Vec<Block>, LedgerError> {
        debug!("正在获取区块范围: {} -> {}", start_height, end_height);
        let conn = self.conn.lock()?;
        
        let mut stmt = conn.prepare(
            "SELECT * FROM blocks WHERE height >= ?1 AND height <= ?2 ORDER BY height ASC"
        ).map_err(LedgerError::DatabaseError)?;
        
        let blocks = stmt.query_map(
            [start_height.to_string(), end_height.to_string()], 
            |row| {
                let header = BlockHeader {
                    block_hash: to_hash(hex::decode(row.get::<_, String>(0)?)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                        ))?)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                        ))?,
                    parent_hash: to_hash(hex::decode(row.get::<_, String>(1)?)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            1,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                        ))?)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            1,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                        ))?,
                    height: row.get(2)?,
                    timestamp: chrono::DateTime::from_timestamp(
                        row.get::<_, String>(3)?
                            .parse::<i64>()
                            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                                3,
                                rusqlite::types::Type::Text,
                                Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                            ))?,
                        0
                    ).ok_or_else(|| rusqlite::Error::FromSqlConversionFailure(
                        3,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid timestamp"))
                    ))?,
                    merkle_root: to_hash(hex::decode(row.get::<_, String>(4)?)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            4,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                        ))?)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            4,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                        ))?,
                    validator: row.get(5)?,
                    signature: SignatureWrapper::from_bytes(&hex::decode(row.get::<_, String>(6)?)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            6,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                        ))?)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            6,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                        ))?,
                    block_number: row.get(2)?,
                    previous_block_hash: to_hash(hex::decode(row.get::<_, String>(1)?)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            1,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                        ))?)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            1,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                        ))?,
                };
                
                let block_hash = hex::encode(&header.block_hash);
                let transactions = self.load_block_transactions(&block_hash)?;
                
                Ok(Block {
                    header,
                    transactions,
                })
            }
        ).map_err(LedgerError::DatabaseError)?;
    
        let result = blocks.collect::<Result<Vec<_>, _>>()
            .map_err(LedgerError::DatabaseError)?;
        
        debug!("获取到 {} 个区块", result.len());
        Ok(result)
    }

}



