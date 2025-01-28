use crate::types::ledger::BlockManagement;
use crate::types::transaction::Transaction;
use crate::types::block::{Block, BlockHeader, calculate_block_hash};
use crate::crypto::hash::Hash;
use crate::crypto::signature::SignatureWrapper;
use super::error::LedgerError;
use super::db::operation::BlockOperations;
use super::merkletree::MerkleTreeManager;
use std::sync::Arc;
use sqlx::SqlitePool;
use log::{debug, error};
use std::sync::Mutex;

pub struct BlockManager {
    db_ops: BlockOperations,
    pool: Arc<SqlitePool>,
    merkle_manager: Arc<Mutex<MerkleTreeManager>>,  
}

impl BlockManager {
    pub fn new(pool: Arc<SqlitePool>, merkle_manager: Arc<Mutex<MerkleTreeManager>>) -> Result<Self, LedgerError> {
        let db_ops = BlockOperations::new(pool.clone());
        Ok(Self {
            db_ops,
            pool,
            merkle_manager,
        })
    }

    // 添加新方法：创建区块时构建默克尔树
    pub async fn create_block(&self, transactions: &[Transaction]) -> Result<Block, LedgerError> {
        debug!("正在创建新区块，交易数量: {}", transactions.len());
        
        // 获取最新区块信息
        let latest_block = self.db_ops.get_latest_block().await?;
        let current_height = latest_block
            .as_ref()
            .map(|b| b.header.height + 1)
            .unwrap_or(0);
        let parent_hash = latest_block
            .as_ref()
            .map(|b| b.header.block_hash.clone())
            .unwrap_or_default();
        
        // 构建默克尔树
        let merkle_root = {
            let mut merkle_manager = self.merkle_manager.lock().map_err(|e| {
                error!("获取默克尔树管理器锁失败: {}", e);
                LedgerError::LockError(e.to_string())
            })?;
            merkle_manager.build_merkle_tree(transactions).await?
        };
    
        // 创建区块头
        let header = BlockHeader {
            parent_hash: parent_hash.clone(),
            height: current_height,
            timestamp: chrono::Utc::now(),
            merkle_root,
            validator: String::new(), // TODO: 添加验证者信息
            signature: SignatureWrapper::from_bytes(&[0u8; 65])
                .map_err(|e| LedgerError::SignatureError(e.to_string()))?, // TODO: 添加签名
            block_hash: Hash::default(), // 临时值，后面会更新
            block_number: current_height,
            previous_block_hash: parent_hash,
        };
    
        // 计算区块哈希
        let block_hash = calculate_block_hash(&header).map_err(|e| LedgerError::BlockError(e.to_string()))?;
        
        // 创建完整区块
        let mut block = Block {
            header,
            transactions: transactions.to_vec(),
        };

        // 保存区块到数据库
        self.db_ops.insert_block(&block).await?;
        
        debug!("新区块创建成功: height={}, hash={}", 
            block.header.height,
            hex::encode(block.header.block_hash.as_bytes())
        );
        
        // 更新区块哈希
        block.header.block_hash = block_hash;
        
        debug!("新区块创建成功: height={}, hash={}", 
            block.header.height,
            hex::encode(block.header.block_hash.as_bytes())
        );
        
        // 发送区块创建事件
        self.send_block_created_event(&block);
        
        Ok(block)
    }

    fn send_block_created_event(&self, block: &Block) {
        // TODO: 实现事件发送逻辑
        debug!("区块创建事件已发送: height={}, hash={}", 
            block.header.height,
            hex::encode(block.header.block_hash.as_bytes())
        );
    }

    pub async fn verify_block(&self, block: &Block) -> Result<bool, LedgerError> {
        // 1. 验证区块哈希
        let computed_hash = calculate_block_hash(&block.header).map_err(|e| LedgerError::BlockError(e.to_string()))?;
        if computed_hash != block.header.block_hash {
            error!("区块哈希验证失败");
            return Ok(false);
        }
    
        // 2. 验证默克尔树
        if !self.verify_block_merkle_tree(block).await? {
            error!("默克尔树验证失败");
            return Ok(false);
        }
    
        // 3. 验证区块签名
        let block_bytes = serde_json::to_vec(&block.header)
            .map_err(|e| LedgerError::SerializationError(e))?;
        
        if !block.header.signature.verify(
            &block_bytes,
            &block.header.validator
        ).map_err(|e| LedgerError::SignatureError(e))? {
            error!("区块签名验证失败");
            return Ok(false);
        }
    
        debug!("区块验证成功: height={}, hash={}", 
            block.header.height,
            hex::encode(block.header.block_hash.as_bytes())
        );
    
        Ok(true)
    }

    // 添加新方法：验证区块的默克尔树
    pub async fn verify_block_merkle_tree(&self, block: &Block) -> Result<bool, LedgerError> {
        let mut merkle_manager = self.merkle_manager.lock().map_err(|e| {
            error!("获取默克尔树管理器锁失败: {}", e);
            LedgerError::LockError(e.to_string())
        })?;
        
        // 如果交易列表为空，直接验证是否为默认哈希
        if block.transactions.is_empty() {
            return Ok(block.header.merkle_root == Hash::default());
        }
    
        // 验证默克尔树根
        let computed_root = merkle_manager.verify_merkle_root(
            &block.transactions,
            &block.header.merkle_root
        ).await?;
        
        Ok(computed_root)
    }

    pub async fn get_block_management(&self) -> Result<BlockManagement, LedgerError> {
        debug!("正在获取区块管理信息...");
        let latest_block = self.db_ops.get_latest_block().await?;
        let blocks = self.db_ops.get_blocks_by_range(0, u64::MAX).await?;
        let height = latest_block.as_ref().map(|b| b.header.height).unwrap_or(0);
        
        debug!("获取到区块数量: {}, 当前高度: {}", blocks.len(), height);
        
        Ok(BlockManagement {
            blocks,
            latest_block,
            block_height: height,
        })
    }

    pub async fn sync_to_db(&self, management: &BlockManagement) -> Result<(), LedgerError> {
        debug!("正在同步区块管理信息到数据库...");
        
        // 开始事务
        let transaction = self.pool.begin().await?;
        
        let result = async {
            // 同步所有区块
            for block in &management.blocks {
                self.db_ops.insert_block(block).await?;
            }
            Ok::<(), LedgerError>(())
        }.await;

        match result {
            Ok(_) => {
                transaction.commit().await?;
                debug!("区块管理信息同步完成");
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