use rusqlite::{Connection, OptionalExtension, params};
use std::sync::{Arc, Mutex};
use log::{debug, info};
use super::super::error::LedgerError;
use crate::types::wallet::Wallet;
use crate::types::transaction::Transaction;
use crate::types::amount::Amount;
use crate::types::block::{Block, BlockHeader};

/// 数据库管理器
pub struct DatabaseManager {
    conn: Arc<Mutex<Connection>>,
}

impl DatabaseManager {
    /// 创建新的数据库管理器实例
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// 初始化数据库表
    pub fn initialize_tables(&self) -> Result<(), LedgerError> {
        debug!("正在初始化数据库表...");
        let conn = self.conn.lock()?;

        // 创建钱包表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS wallets (
                address TEXT PRIMARY KEY,
                balance TEXT NOT NULL,
                nonce INTEGER NOT NULL
            )",
            [],
        ).map_err(LedgerError::DatabaseError)?;

        // 创建交易表
        conn.execute(
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
            )",
            [],
        ).map_err(LedgerError::DatabaseError)?;

        // 创建区块表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS blocks (
                block_hash TEXT PRIMARY KEY,
                parent_hash TEXT NOT NULL,
                height INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                merkle_root TEXT NOT NULL,
                validator TEXT NOT NULL,
                signature TEXT NOT NULL
            )",
            [],
        ).map_err(LedgerError::DatabaseError)?;

        // 创建索引
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_transactions_from ON transactions(from_address)",
            [],
        ).map_err(LedgerError::DatabaseError)?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_transactions_to ON transactions(to_address)",
            [],
        ).map_err(LedgerError::DatabaseError)?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_transactions_block ON transactions(block_hash)",
            [],
        ).map_err(LedgerError::DatabaseError)?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_blocks_height ON blocks(height)",
            [],
        ).map_err(LedgerError::DatabaseError)?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_blocks_parent ON blocks(parent_hash)",
            [],
        ).map_err(LedgerError::DatabaseError)?;

        info!("数据库表初始化完成");
        Ok(())
    }
}

/// 钱包数据库操作
pub struct WalletOperations {
    conn: Arc<Mutex<Connection>>,
}

/// 交易数据库操作
pub struct TransactionOperations {
    conn: Arc<Mutex<Connection>>,
}

/// 区块数据库操作
pub struct BlockOperations {
    conn: Arc<Mutex<Connection>>,
}

impl WalletOperations {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn insert_wallet(&self, wallet: &Wallet) -> Result<(), LedgerError> {
        debug!("正在插入钱包记录: {}", wallet.address);
        let conn = self.conn.lock()?;
        
        conn.execute(
            "INSERT INTO wallets (address, balance, nonce) VALUES (?1, ?2, ?3)",
            params![wallet.address, wallet.balance.to_string(), wallet.nonce],
        ).map_err(LedgerError::DatabaseError)?;
        
        debug!("钱包记录插入成功");
        Ok(())
    }

    pub fn get_wallet(&self, address: &str) -> Result<Wallet, LedgerError> {
        debug!("正在查询钱包: {}", address);
        let conn = self.conn.lock()?;
        
        let mut stmt = conn.prepare(
            "SELECT address, balance, nonce FROM wallets WHERE address = ?1"
        ).map_err(LedgerError::DatabaseError)?;
        
        let wallet = stmt.query_row([address], |row| {
            Ok(Wallet {
                address: row.get(0)?,
                balance: row.get(1)?,
                nonce: row.get(2)?,
            })
        }).map_err(LedgerError::DatabaseError)?;
        
        debug!("钱包查询成功");
        Ok(wallet)
    }

    /// 更新钱包余额,转账成功后的余额
    pub fn update_wallet_balance(&self, address: &str, amount: Amount) -> Result<(), LedgerError> {
        debug!("正在更新钱包余额: {} -> {}", address, amount.to_string());
        let conn = self.conn.lock()?;
        
        conn.execute(
            "UPDATE wallets SET balance = ?1 WHERE address = ?2",
            [&amount.to_string(), address],
        ).map_err(LedgerError::DatabaseError)?;
        
        debug!("钱包余额更新成功");
        Ok(())
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
                transaction_type: serde_json::from_str(&row.get::<_, String>(1)?).unwrap(),
                from: row.get(2)?,
                to: row.get(3)?,
                transfer_amount: Amount::from_str(&row.get::<_, String>(4)?).unwrap(),
                nonce: row.get::<_, String>(5)?.parse().unwrap(),
                signature: hex::decode(row.get::<_, String>(6)?).unwrap(),
                timestamp: chrono::DateTime::from_timestamp(
                    row.get::<_, String>(7)?.parse().unwrap(), 0
                ).unwrap(),
                fee: Amount::from_str(&row.get::<_, String>(8)?).unwrap(),
                transaction_hash: hex::decode(row.get::<_, String>(0)?).unwrap(),
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
                transaction_type: serde_json::from_str(&row.get::<_, String>(1)?).unwrap(),
                from: row.get(2)?,
                to: row.get(3)?,
                transfer_amount: Amount::from_str(&row.get::<_, String>(4)?).unwrap(),
                nonce: row.get::<_, String>(5)?.parse().unwrap(),
                signature: hex::decode(row.get::<_, String>(6)?).unwrap(),
                timestamp: chrono::DateTime::from_timestamp(
                    row.get::<_, String>(7)?.parse().unwrap(), 0
                ).unwrap(),
                fee: Amount::from_str(&row.get::<_, String>(8)?).unwrap(),
                transaction_hash: hex::decode(row.get::<_, String>(0)?).unwrap(),
            })
        })?;
    
        transactions.collect()
    }

    pub fn insert_block(&self, block: &Block) -> Result<(), LedgerError> {
        debug!("正在插入区块: {:?}", hex::encode(&block.header.block_hash));
        let mut conn = self.conn.lock()?;
    
        // 开始事务
        let transaction = conn.transaction().map_err(LedgerError::DatabaseError)?;
        
        // 1. 验证所有交易是否存在且状态正确
        for block_transaction in &block.transactions {
            let hash = hex::encode(&block_transaction.transaction_hash);
            let status: Option<String> = transaction.query_row(
                "SELECT status FROM transactions WHERE hash = ?1",
                [&hash],
                |row| row.get(0)
            ).optional().map_err(LedgerError::DatabaseError)?;

            match status {
                None => {
                    // 交易不存在，先插入交易记录
                    transaction.execute(
                        "INSERT INTO transactions (
                            hash, transaction_type, from_address, to_address, transfer_amount,
                            nonce, signature, timestamp, fee, block_hash, status
                        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                        params![
                            &hash,
                            &format!("{:?}", block_transaction.transaction_type),
                            &block_transaction.from,
                            &block_transaction.to,
                            &block_transaction.transfer_amount,
                            &block_transaction.nonce,
                            &hex::encode(&block_transaction.signature),
                            &block_transaction.timestamp.timestamp(),
                            &block_transaction.fee,
                            "",
                            "pending"
                        ],
                    ).map_err(LedgerError::DatabaseError)?;
                },
                Some(current_status) => {
                    if current_status == "confirmed" {
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

        // 3. 更新所有交易状态
        let block_hash = hex::encode(&block.header.block_hash);
        for block_transaction in &block.transactions {
            transaction.execute(  // 使用 transaction 变量执行 SQL 语句
                "UPDATE transactions 
                SET status = 'confirmed', block_hash = ?1 
                WHERE hash = ?2 AND status = 'pending'",
                [&block_hash, &hex::encode(&block_transaction.transaction_hash)],
            ).map_err(LedgerError::DatabaseError)?;
        }

        // 4. 提交事务
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
                block_hash: hex::decode(row.get::<_, String>(0)?).unwrap(),
                parent_hash: hex::decode(row.get::<_, String>(1)?).unwrap(),
                height: row.get(2)?,
                timestamp: chrono::DateTime::from_timestamp(
                    row.get::<_, String>(3)?.parse().unwrap(), 0
                ).unwrap(),
                merkle_root: hex::decode(row.get::<_, String>(4)?).unwrap(),
                validator: row.get(5)?,
                signature: hex::decode(row.get::<_, String>(6)?).unwrap(),
                block_number: row.get(2)?,
                previous_block_hash: hex::decode(row.get::<_, String>(1)?).unwrap(),
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
                block_hash: hex::decode(row.get::<_, String>(0)?).unwrap(),
                parent_hash: hex::decode(row.get::<_, String>(1)?).unwrap(),
                height: row.get(2)?,
                timestamp: chrono::DateTime::from_timestamp(
                    row.get::<_, String>(3)?.parse().unwrap(), 0
                ).unwrap(),
                merkle_root: hex::decode(row.get::<_, String>(4)?).unwrap(),
                validator: row.get(5)?,
                signature: hex::decode(row.get::<_, String>(6)?).unwrap(),
                block_number: row.get(2)?,
                previous_block_hash: hex::decode(row.get::<_, String>(1)?).unwrap(),
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
                block_hash: hex::decode(row.get::<_, String>(0)?).unwrap(),
                parent_hash: hex::decode(row.get::<_, String>(1)?).unwrap(),
                height: row.get(2)?,
                timestamp: chrono::DateTime::from_timestamp(
                    row.get::<_, String>(3)?.parse().unwrap(), 0
                ).unwrap(),
                merkle_root: hex::decode(row.get::<_, String>(4)?).unwrap(),
                validator: row.get(5)?,
                signature: hex::decode(row.get::<_, String>(6)?).unwrap(),
                block_number: row.get(2)?,
                previous_block_hash: hex::decode(row.get::<_, String>(1)?).unwrap(),
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
}



