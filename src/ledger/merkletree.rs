use super::error::LedgerError;
use crate::crypto::hash::{to_hash, Hash};
use crate::types::transaction::TransactionDetail;
use log::{debug, error};
use merkletree::hash::Algorithm;
use merkletree::merkle::MerkleTree;
use merkletree::store::VecStore;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::hash::Hasher;
use std::sync::Arc;

/// 默克尔树管理器
pub struct MerkleTreeManager {
    pool: Arc<SqlitePool>,
    tree: Option<MerkleTree<[u8; 32], Hash, VecStore<[u8; 32]>>>,
}

impl MerkleTreeManager {
    /// 创建新的默克尔树管理器
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool, tree: None }
    }

    /// 从交易列表构建默克尔树
    pub async fn build_merkle_tree(
        &mut self,
        transactions: &[TransactionDetail],
    ) -> Result<Hash, LedgerError> {
        debug!("正在构建默克尔树，交易数量: {}", transactions.len());

        let transaction_data: Vec<[u8; 32]> = transactions
            .iter()
            .map(|transaction| {
                let mut hasher = Hash::default();
                hasher.write(transaction.transaction_hash.as_ref());
                hasher.hash()
            })
            .collect();

        if transaction_data.is_empty() {
            debug!("交易列表为空，返回默认哈希");
            return Ok(Hash::default());
        }

        let tree = MerkleTree::<[u8; 32], Hash, VecStore<[u8; 32]>>::new(transaction_data)
            .map_err(|e| {
                error!("构建默克尔树失败: {}", e);
                LedgerError::MerkleTreeError(e.to_string())
            })?;

        let root = Hash::from_slice(&tree.root()).map_err(|e| {
            error!("获取默克尔树根哈希失败: {}", e);
            LedgerError::MerkleTreeError(e.to_string())
        })?;

        self.tree = Some(tree);

        // 保存默克尔树根到数据库
        self.save_merkle_root(&root).await?;

        debug!("默克尔树构建完成，根哈希: {}", hex::encode(root.as_bytes()));
        Ok(root)
    }

    /// 保存默克尔树根到数据库
    async fn save_merkle_root(&self, root: &Hash) -> Result<(), LedgerError> {
        debug!("正在保存默克尔树根到数据库");

        sqlx::query("INSERT INTO merkle_roots (root_hash, timestamp) VALUES ($1, $2)")
            .bind(hex::encode(root.as_bytes()))
            .bind(chrono::Utc::now().timestamp())
            .execute(&*self.pool)
            .await
            .map_err(|e| {
                error!("保存默克尔树根失败: {}", e);
                LedgerError::DatabaseError(e)
            })?;

        debug!("默克尔树根保存成功");
        Ok(())
    }

    /// 生成交易的默克尔证明
    pub async fn generate_proof(
        &self,
        transaction: &TransactionDetail,
    ) -> Result<MerkleProof, LedgerError> {
        let tree = self
            .tree
            .as_ref()
            .ok_or_else(|| LedgerError::MerkleTreeError("默克尔树未初始化".to_string()))?;

        let mut hasher = Hash::default();
        hasher.write(transaction.transaction_hash.as_ref());
        let hash = hasher.hash();

        // 查找交易哈希对应的索引
        let mut index = None;
        for i in 0..tree.leafs() {
            if let Ok(element) = tree.read_at(i) {
                if element == hash.as_ref() {
                    index = Some(i);
                    break;
                }
            }
        }

        let index = index.ok_or_else(|| {
            error!("交易未找到: {:?}", transaction.transaction_hash);
            LedgerError::MerkleTreeError("交易未找到".to_string())
        })?;

        let proof = tree.gen_proof(index).map_err(|e| {
            error!("生成默克尔证明失败: {}", e);
            LedgerError::MerkleTreeError(e.to_string())
        })?;

        let proof_hashes = proof
            .lemma()
            .iter()
            .map(|h| Hash::from_slice(h))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                error!("处理证明哈希失败: {}", e);
                LedgerError::MerkleTreeError(e.to_string())
            })?;

        let merkle_proof = MerkleProof {
            proof_hashes,
            root_hash: Hash::from_slice(&tree.root()).map_err(|e| {
                error!("获取根哈希失败: {}", e);
                LedgerError::MerkleTreeError(e.to_string())
            })?,
        };

        // 保存默克尔证明到数据库
        self.save_merkle_proof(transaction, &merkle_proof).await?;

        Ok(merkle_proof)
    }

    /// 保存默克尔证明到数据库
    async fn save_merkle_proof(
        &self,
        transaction: &TransactionDetail,
        proof: &MerkleProof,
    ) -> Result<(), LedgerError> {
        debug!("正在保存默克尔证明到数据库");

        let proof_data = serde_json::to_string(&proof).map_err(LedgerError::SerializationError)?;

        sqlx::query(
            "INSERT INTO merkle_proofs (
                transaction_hash, 
                root_hash, 
                proof_data, 
                timestamp
            ) VALUES ($1, $2, $3, $4)",
        )
        .bind(hex::encode(&transaction.transaction_hash))
        .bind(hex::encode(proof.root_hash.as_bytes()))
        .bind(&proof_data)
        .bind(chrono::Utc::now().timestamp())
        .execute(&*self.pool)
        .await
        .map_err(|e| {
            error!("保存默克尔证明失败: {}", e);
            LedgerError::DatabaseError(e)
        })?;

        debug!("默克尔证明保存成功");
        Ok(())
    }

    /// 验证默克尔树根
    pub async fn verify_merkle_root(
        &mut self,
        transactions: &[TransactionDetail],
        expected_root: &Hash,
    ) -> Result<bool, LedgerError> {
        // 构建交易哈希列表
        let transaction_data: Vec<[u8; 32]> = transactions
            .iter()
            .map(|transaction| {
                let mut hasher = Hash::default();
                hasher.write(transaction.transaction_hash.as_ref());
                hasher.hash()
            })
            .collect();

        // 构建临时树进行验证
        let tree = MerkleTree::<[u8; 32], Hash, VecStore<[u8; 32]>>::new(transaction_data)
            .map_err(|e| {
                error!("构建默克尔树失败: {}", e);
                LedgerError::MerkleTreeError(e.to_string())
            })?;

        let root = Hash::from_slice(&tree.root()).map_err(|e| {
            error!("获取默克尔树根哈希失败: {}", e);
            LedgerError::MerkleTreeError(e.to_string())
        })?;

        Ok(root == *expected_root)
    }
}

/// 默克尔证明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub proof_hashes: Vec<Hash>,
    pub root_hash: Hash,

}

impl MerkleProof {
    /// 验证默克尔证明
    fn verify(&self, transaction_hash: &[u8]) -> Result<bool, String> {
        let mut current = Hash::from_slice(transaction_hash).map_err(|e| e.to_string())?;
        
        for proof_hash in &self.proof_hashes {
            let combined = if current.as_ref() <= proof_hash.as_ref() {
                [current.as_ref(), proof_hash.as_ref()].concat()
            } else {
                [proof_hash.as_ref(), current.as_ref()].concat()
            };
            
            // 修复类型不匹配问题
            let hash_bytes = to_hash(combined.as_slice().to_vec()).map_err(|e| e.to_string())?;
            current = Hash::from_slice(hash_bytes.as_ref()).map_err(|e| e.to_string())?;
        }

        Ok(current == self.root_hash)
    }

}









