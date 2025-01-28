
use sqlx::sqlite::SqlitePool;
use std::sync::Arc;
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
                return Err(LedgerError::DatabaseError(e));
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
                return Err(LedgerError::DatabaseError(e));
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
                return Err(LedgerError::DatabaseError(e));
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
                return Err(LedgerError::DatabaseError(e));
            }
        };
        
        // 创建默克尔树表
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS merkle_roots (
                root_hash TEXT PRIMARY KEY,
                block_hash TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                height INTEGER NOT NULL,
                FOREIGN KEY (block_hash) REFERENCES blocks(block_hash)
            )"
        )
        .execute(pool)
        .await?;
        
        // 创建默克尔证明表
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS merkle_proofs (
                transaction_hash TEXT PRIMARY KEY,
                root_hash TEXT NOT NULL,
                proof_data TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                FOREIGN KEY (transaction_hash) REFERENCES transactions(hash),
                FOREIGN KEY (root_hash) REFERENCES merkle_roots(root_hash)
            )"
        )
        .execute(pool)
        .await?;

        // 创建索引
        debug!("正在创建索引...");
        for (index_name, query) in [
            ("交易发送方索引", "CREATE INDEX IF NOT EXISTS idx_transactions_from ON transactions(from_address)"),
            ("交易接收方索引", "CREATE INDEX IF NOT EXISTS idx_transactions_to ON transactions(to_address)"),
            ("交易区块索引", "CREATE INDEX IF NOT EXISTS idx_transactions_block ON transactions(block_hash)"),
            ("区块高度索引", "CREATE INDEX IF NOT EXISTS idx_blocks_height ON blocks(height)"),
            ("区块父哈希索引", "CREATE INDEX IF NOT EXISTS idx_blocks_parent ON blocks(parent_hash)"),
            ("钱包历史索引", "CREATE INDEX IF NOT EXISTS idx_wallet_history_address ON wallet_transaction_history(wallet_address)"),
            ("默克尔树根哈希索引", "CREATE INDEX IF NOT EXISTS idx_merkle_roots_root_hash ON merkle_roots(root_hash)"),
            ("默克尔树区块哈希索引", "CREATE INDEX IF NOT EXISTS idx_merkle_roots_block_hash ON merkle_roots(block_hash)"),
            ("默克尔证明交易哈希索引", "CREATE INDEX IF NOT EXISTS idx_merkle_proofs_transaction_hash ON merkle_proofs(transaction_hash)"),
            ("默克尔证明根哈希索引", "CREATE INDEX IF NOT EXISTS idx_merkle_proofs_root_hash ON merkle_proofs(root_hash)")
        ] {
            debug!("正在创建{}...", index_name);
            match sqlx::query(query)
                .execute(pool)
                .await {
                    Ok(_) => debug!("{}创建成功", index_name),
                    Err(e) => {
                        error!("创建{}失败: {}", index_name, e);
                        return Err(LedgerError::DatabaseError(e));
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
                Err(LedgerError::DatabaseError(e))
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
                        .map_err(|e| LedgerError::AmountError(e.to_string()))?,
                    nonce: row.nonce as u64,
                }))
            },
            Ok(None) => {
                debug!("钱包不存在");
                Ok(None)
            },
            Err(e) => {
                error!("查询钱包失败: {}", e);
                Err(LedgerError::DatabaseError(e))
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
                Err(LedgerError::DatabaseError(e))
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
                Err(LedgerError::DatabaseError(e))
            }
        }
    }

    pub async fn update_transaction_history(&self, address: &str, transactions: &[Transaction]) -> Result<(), LedgerError> {
        debug!("正在更新钱包交易历史: {}", address);
        let pool = &*self.pool;
        
        // 开始事务
        let mut transaction = pool.begin().await?;
        
        for transaction_record in transactions {
            let hash = hex::encode(&transaction_record.transaction_hash);
            // 插入或忽略重复记录
            sqlx::query(
                "INSERT OR IGNORE INTO wallet_transaction_history 
                (wallet_address, transaction_hash, timestamp) 
                VALUES ($1, $2, $3)"
            )
            .bind(address)
            .bind(&hash)
            .bind(transaction_record.timestamp.timestamp())
            .execute(&mut *transaction)
            .await?;
        }
        
        // 提交事务
        transaction.commit().await?;
        
        debug!("交易历史更新成功");
        Ok(())
    }

    pub async fn get_transaction_history(&self, address: &str) -> Result<Vec<Transaction>, LedgerError> {
        debug!("正在获取钱包交易历史: {}", address);
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
            block_hash: Option<String>,
            status: String,
        }
    
        // 执行查询
        let rows = sqlx::query_as::<_, TransactionRow>(
            "SELECT t.* FROM transactions t
            INNER JOIN wallet_transaction_history wth ON t.hash = wth.transaction_hash
            WHERE wth.wallet_address = $1
            ORDER BY wth.timestamp DESC"
        )
        .bind(address)
        .fetch_all(pool)
        .await?;
    
        // 转换查询结果
        let transactions = rows.into_iter()
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
    
        debug!("获取到 {} 笔交易历史", transactions.len());
        Ok(transactions)
    }

}

/// 交易数据库操作
pub struct TransactionOperations {
    pool: Arc<SqlitePool>,
}

impl TransactionOperations {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    /// 插入新的交易记录
    pub async fn insert_transaction(&self, transaction: Transaction) -> Result<(), LedgerError> {
        debug!("正在插入交易记录: {:?}", transaction.transaction_hash);
        let pool = &*self.pool;
        
        sqlx::query(
            "INSERT INTO transactions (
                hash, transaction_type, from_address, to_address, transfer_amount, 
                nonce, signature, timestamp, fee, block_hash, status
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
        )
        .bind(&hex::encode(&transaction.transaction_hash))
        .bind(&format!("{:?}", transaction.transaction_type))
        .bind(&transaction.from)
        .bind(&transaction.to)
        .bind(&transaction.transfer_amount.to_string())
        .bind(transaction.nonce as i64)
        .bind(&hex::encode(&transaction.signature))
        .bind(transaction.timestamp.timestamp())
        .bind(&transaction.fee.to_string())
        .bind("")  // 初始区块哈希为空
        .bind("pending")  // 初始状态为待处理
        .execute(pool)
        .await?;
        
        debug!("交易记录插入成功");
        Ok(())
    }

    /// 获取待处理的交易列表
    pub async fn get_pending_transactions(&self) -> Result<Vec<Transaction>, LedgerError> {
        debug!("正在获取待处理交易");
        let pool = &*self.pool;

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
            block_hash: Option<String>,
            status: String,
        }
        
        let rows = sqlx::query_as::<_, TransactionRow>(
            "SELECT * FROM transactions WHERE status = 'pending' ORDER BY fee DESC"
        )
        .fetch_all(pool)
        .await?;

        let transactions = rows.into_iter()
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
        
        debug!("获取到 {} 笔待处理交易", transactions.len());
        Ok(transactions)
    }

    /// 更新交易状态
    pub async fn update_transaction_status(&self, hash: &str, status: &str) -> Result<(), LedgerError> {
        debug!("正在更新交易状态: {} -> {}", hash, status);
        let pool = &*self.pool;
        
        sqlx::query(
            "UPDATE transactions SET status = $1 WHERE hash = $2"
        )
        .bind(status)
        .bind(hash)
        .execute(pool)
        .await?;
        
        debug!("交易状态更新成功");
        Ok(())
    }

    /// 清理已确认的交易
    pub async fn clean_confirmed_transactions(&self) -> Result<(), LedgerError> {
        debug!("正在清理已确认交易");
        let pool = &*self.pool;
        
        let one_month_ago = chrono::Utc::now()
            .checked_sub_days(chrono::Days::new(30))
            .ok_or_else(|| LedgerError::InvalidData("无法计算一个月前的日期".to_string()))?
            .timestamp();
        
        sqlx::query(
            "DELETE FROM transactions 
            WHERE status = 'confirmed' 
            AND timestamp < $1"
        )
        .bind(one_month_ago)
        .execute(pool)
        .await?;
        
        debug!("已确认交易清理完成");
        Ok(())
    }
}

/// 区块数据库操作
pub struct BlockOperations {
    pool: Arc<SqlitePool>,
}

impl BlockOperations {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    /// 加载区块的交易记录
    async fn load_block_transactions(&self, block_hash: &str) -> Result<Vec<Transaction>, LedgerError> {
        debug!("正在加载区块交易: {}", block_hash);
        let pool = &*self.pool;

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
            block_hash: Option<String>,
            status: String,
        }

        let rows = sqlx::query_as::<_, TransactionRow>(
            "SELECT * FROM transactions WHERE block_hash = $1"
        )
        .bind(block_hash)
        .fetch_all(pool)
        .await?;

        let transactions = rows.into_iter()
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

        debug!("加载到 {} 笔交易", transactions.len());
        Ok(transactions)
    }

    /// 插入新区块
    pub async fn insert_block(&self, block: &Block) -> Result<(), LedgerError> {
        debug!("正在插入区块: {:?}", hex::encode(&block.header.block_hash));
        let pool = &*self.pool;

        let mut transaction = pool.begin().await?;

        // 1. 遍历所有交易，检查并更新状态
        for block_transaction in &block.transactions {
            let hash = hex::encode(&block_transaction.transaction_hash);
            let status: Option<String> = sqlx::query_scalar(
                "SELECT status FROM transactions WHERE hash = $1"
            )
            .bind(&hash)
            .fetch_optional(&mut *transaction)
            .await?;

            match status {
                None => {
                    // 交易不存在，插入交易记录，状态为 confirmed
                    sqlx::query(
                        "INSERT INTO transactions (
                            hash, transaction_type, from_address, to_address, transfer_amount,
                            nonce, signature, timestamp, fee, block_hash, status
                        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'confirmed')"
                    )
                    .bind(&hash)
                    .bind(&format!("{:?}", block_transaction.transaction_type))
                    .bind(&block_transaction.from)
                    .bind(&block_transaction.to)
                    .bind(&block_transaction.transfer_amount.to_string())
                    .bind(block_transaction.nonce as i64)
                    .bind(&hex::encode(&block_transaction.signature))
                    .bind(block_transaction.timestamp.timestamp())
                    .bind(&block_transaction.fee.to_string())
                    .bind(&hex::encode(&block.header.block_hash))
                    .execute(&mut *transaction)
                    .await?;
                },
                Some(current_status) => {
                    if current_status == "pending" {
                        // 交易存在且状态为 pending，更新状态为 confirmed，并关联区块哈希
                        sqlx::query(
                            "UPDATE transactions SET status = 'confirmed', block_hash = $1 WHERE hash = $2"
                        )
                        .bind(&hex::encode(&block.header.block_hash))
                        .bind(&hash)
                        .execute(&mut *transaction)
                        .await?;
                    } else if current_status == "confirmed" {
                        return Err(LedgerError::TransactionAlreadyConfirmed(hash));
                    }
                }
            }
        }

        // 2. 插入区块头
        sqlx::query(
            "INSERT INTO blocks (
                block_hash, parent_hash, height, timestamp,
                merkle_root, validator, signature
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)"
        )
        .bind(&hex::encode(&block.header.block_hash))
        .bind(&hex::encode(&block.header.parent_hash))
        .bind(block.header.height as i64)
        .bind(block.header.timestamp.timestamp())
        .bind(&hex::encode(&block.header.merkle_root))
        .bind(&block.header.validator)
        .bind(&hex::encode(&block.header.signature))
        .execute(&mut *transaction)
        .await?;

        // 3. 提交事务
        transaction.commit().await?;

        debug!("区块及其交易记录插入成功");
        Ok(())
    }

    /// 获取最新区块
    pub async fn get_latest_block(&self) -> Result<Option<Block>, LedgerError> {
        debug!("正在获取最新区块");
        let pool = &*self.pool;

        #[derive(sqlx::FromRow)]
        struct BlockRow {
            block_hash: String,
            parent_hash: String,
            height: i64,
            timestamp: i64,
            merkle_root: String,
            validator: String,
            signature: String,
        }

        let row = sqlx::query_as::<_, BlockRow>(
            "SELECT * FROM blocks ORDER BY height DESC LIMIT 1"
        )
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => {
                let header = BlockHeader {
                    block_hash: to_hash(hex::decode(&row.block_hash)?)
                        .map_err(|e| LedgerError::InvalidData(e.to_string()))?,
                    parent_hash: to_hash(hex::decode(&row.parent_hash)?)
                        .map_err(|e| LedgerError::InvalidData(e.to_string()))?,
                    height: row.height as u64,
                    timestamp: chrono::DateTime::from_timestamp(row.timestamp, 0)
                        .ok_or_else(|| LedgerError::TimestampError("无效的时间戳".to_string()))?,
                    merkle_root: to_hash(hex::decode(&row.merkle_root)?)
                        .map_err(|e| LedgerError::InvalidData(e.to_string()))?,
                    validator: row.validator,
                    signature: SignatureWrapper::from_bytes(&hex::decode(&row.signature)?)
                        .map_err(|e| LedgerError::SignatureError(e.to_string()))?,
                    block_number: row.height as u64,
                    previous_block_hash: to_hash(hex::decode(&row.parent_hash)?)
                        .map_err(|e| LedgerError::InvalidData(e.to_string()))?,
                };

                let transactions = self.load_block_transactions(&row.block_hash).await?;

                Ok(Some(Block {
                    header,
                    transactions,
                }))
            },
            None => Ok(None)
        }
    }

    /// 通过哈希获取区块
    pub async fn get_block_by_hash(&self, hash: &str) -> Result<Option<Block>, LedgerError> {
        debug!("正在通过哈希获取区块: {}", hash);
        let pool = &*self.pool;

        #[derive(sqlx::FromRow)]
        struct BlockRow {
            block_hash: String,
            parent_hash: String,
            height: i64,
            timestamp: i64,
            merkle_root: String,
            validator: String,
            signature: String,
        }

        let row = sqlx::query_as::<_, BlockRow>(
            "SELECT * FROM blocks WHERE block_hash = $1"
        )
        .bind(hash)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => {
                let header = BlockHeader {
                    block_hash: to_hash(hex::decode(&row.block_hash)?)
                        .map_err(|e| LedgerError::InvalidData(e.to_string()))?,
                    parent_hash: to_hash(hex::decode(&row.parent_hash)?)
                        .map_err(|e| LedgerError::InvalidData(e.to_string()))?,
                    height: row.height as u64,
                    timestamp: chrono::DateTime::from_timestamp(row.timestamp, 0)
                        .ok_or_else(|| LedgerError::TimestampError("无效的时间戳".to_string()))?,
                    merkle_root: to_hash(hex::decode(&row.merkle_root)?)
                        .map_err(|e| LedgerError::InvalidData(e.to_string()))?,
                    validator: row.validator,
                    signature: SignatureWrapper::from_bytes(&hex::decode(&row.signature)?)
                        .map_err(|e| LedgerError::SignatureError(e.to_string()))?,
                    block_number: row.height as u64,
                    previous_block_hash: to_hash(hex::decode(&row.parent_hash)?)
                        .map_err(|e| LedgerError::InvalidData(e.to_string()))?,
                };

                let transactions = self.load_block_transactions(&row.block_hash).await?;

                Ok(Some(Block {
                    header,
                    transactions,
                }))
            },
            None => Ok(None)
        }
    }

    /// 获取区块高度
    pub async fn get_block_height(&self) -> Result<u64, LedgerError> {
        let pool = &*self.pool;
        let height: Option<i64> = sqlx::query_scalar(
            "SELECT MAX(height) FROM blocks"
        )
        .fetch_optional(pool)
        .await?;
        
        Ok(height.unwrap_or(0) as u64)
    }

    /// 通过高度获取区块
    pub async fn get_block_by_height(&self, height: u64) -> Result<Option<Block>, LedgerError> {
        debug!("正在通过高度获取区块: {}", height);
        let pool = &*self.pool;

        #[derive(sqlx::FromRow)]
        struct BlockRow {
            block_hash: String,
            parent_hash: String,
            height: i64,
            timestamp: i64,
            merkle_root: String,
            validator: String,
            signature: String,
        }

        let row = sqlx::query_as::<_, BlockRow>(
            "SELECT * FROM blocks WHERE height = $1"
        )
        .bind(height as i64)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => {
                let header = BlockHeader {
                    block_hash: to_hash(hex::decode(&row.block_hash)?)
                        .map_err(|e| LedgerError::InvalidData(e.to_string()))?,
                    parent_hash: to_hash(hex::decode(&row.parent_hash)?)
                        .map_err(|e| LedgerError::InvalidData(e.to_string()))?,
                    height: row.height as u64,
                    timestamp: chrono::DateTime::from_timestamp(row.timestamp, 0)
                        .ok_or_else(|| LedgerError::TimestampError("无效的时间戳".to_string()))?,
                    merkle_root: to_hash(hex::decode(&row.merkle_root)?)
                        .map_err(|e| LedgerError::InvalidData(e.to_string()))?,
                    validator: row.validator,
                    signature: SignatureWrapper::from_bytes(&hex::decode(&row.signature)?)
                        .map_err(|e| LedgerError::SignatureError(e.to_string()))?,
                    block_number: row.height as u64,
                    previous_block_hash: to_hash(hex::decode(&row.parent_hash)?)
                        .map_err(|e| LedgerError::InvalidData(e.to_string()))?,
                };

                let transactions = self.load_block_transactions(&row.block_hash).await?;

                Ok(Some(Block {
                    header,
                    transactions,
                }))
            },
            None => Ok(None)
        }
    }

    /// 获取指定高度范围内的区块
    pub async fn get_blocks_by_range(&self, start_height: u64, end_height: u64) -> Result<Vec<Block>, LedgerError> {
        debug!("正在获取区块范围: {} -> {}", start_height, end_height);
        let pool = &*self.pool;

        #[derive(sqlx::FromRow)]
        struct BlockRow {
            block_hash: String,
            parent_hash: String,
            height: i64,
            timestamp: i64,
            merkle_root: String,
            validator: String,
            signature: String,
        }

        let rows = sqlx::query_as::<_, BlockRow>(
            "SELECT * FROM blocks WHERE height >= $1 AND height <= $2 ORDER BY height ASC"
        )
        .bind(start_height as i64)
        .bind(end_height as i64)
        .fetch_all(pool)
        .await?;

        let mut blocks = Vec::new();
        for row in rows {
            let header = BlockHeader {
                block_hash: to_hash(hex::decode(&row.block_hash)?)
                    .map_err(|e| LedgerError::InvalidData(e.to_string()))?,
                parent_hash: to_hash(hex::decode(&row.parent_hash)?)
                    .map_err(|e| LedgerError::InvalidData(e.to_string()))?,
                height: row.height as u64,
                timestamp: chrono::DateTime::from_timestamp(row.timestamp, 0)
                    .ok_or_else(|| LedgerError::TimestampError("无效的时间戳".to_string()))?,
                merkle_root: to_hash(hex::decode(&row.merkle_root)?)
                    .map_err(|e| LedgerError::InvalidData(e.to_string()))?,
                validator: row.validator,
                signature: SignatureWrapper::from_bytes(&hex::decode(&row.signature)?)
                    .map_err(|e| LedgerError::SignatureError(e.to_string()))?,
                block_number: row.height as u64,
                previous_block_hash: to_hash(hex::decode(&row.parent_hash)?)
                    .map_err(|e| LedgerError::InvalidData(e.to_string()))?,
            };

            let transactions = self.load_block_transactions(&row.block_hash).await?;

            blocks.push(Block {
                header,
                transactions,
            });
        }

        debug!("获取到 {} 个区块", blocks.len());
        Ok(blocks)
    }
}



