use crate::types::ledger::BlockManagement;
use super::error::LedgerError;
use super::db::operation::BlockOperations;
use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use log::debug;

pub struct BlockManager {
    db_ops: BlockOperations,
    conn: Arc<Mutex<Connection>>,
}

impl BlockManager {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Result<Self, LedgerError> {
        let db_ops = BlockOperations::new(conn.clone());
        Ok(Self {
            db_ops,
            conn: conn.clone(),
        })
    }

    pub async fn get_block_management(&self) -> Result<BlockManagement, LedgerError> {
        debug!("正在获取区块管理信息...");
        let latest_block = self.db_ops.get_latest_block()?;
        let blocks = self.db_ops.get_blocks_by_range(0, u64::MAX)?;
        let height = latest_block.as_ref().map(|b| b.header.height).unwrap_or(0);
        
        Ok(BlockManagement {
            blocks,
            latest_block,
            block_height: height,
        })
    }

    pub async fn sync_to_db(&self, management: &BlockManagement) -> Result<(), LedgerError> {
        debug!("正在同步区块管理信息到数据库...");
        // 同步所有区块
        for block in &management.blocks {
            self.db_ops.insert_block(block)?;
        }
        
        debug!("区块管理信息同步完成");
        Ok(())
    }
}