use crate::types::transaction::Transaction;
use crate::crypto::hash::{Hash, to_hash};
use super::error::LedgerError;
use log::debug;
use merkletree::merkle::MerkleTree;
use merkletree::store::VecStore;
use merkletree::hash::Algorithm;
use std::hash::Hasher;
use serde::{Serialize, Deserialize};

/// 默克尔树管理器
pub struct MerkleTreeManager {
    tree: Option<MerkleTree<[u8; 32], Hash, VecStore<[u8; 32]>>>
}

impl MerkleTreeManager {
    /// 创建新的默克尔树管理器
    pub fn new() -> Self {
        Self { tree: None }
    }

    /// 从交易列表构建默克尔树
    pub fn build_merkle_tree(&mut self, transactions: &[Transaction]) -> Result<Hash, LedgerError> {
        debug!("正在构建默克尔树，交易数量: {}", transactions.len());
        
        let transaction_data: Vec<[u8; 32]> = transactions
            .iter()
            .map(|tx| {
                let mut hasher = Hash::default();
                hasher.write(tx.transaction_hash.as_ref());
                hasher.hash()
            })
            .collect();

        if transaction_data.is_empty() {
            return Ok(Hash::default());
        }

        let tree = MerkleTree::<[u8; 32], Hash, VecStore<[u8; 32]>>::new(transaction_data)
            .map_err(|e| LedgerError::MerkleTreeError(e.to_string()))?;
            
        let root = Hash::from_slice(&tree.root())
            .map_err(|e| LedgerError::MerkleTreeError(e.to_string()))?;
        self.tree = Some(tree);
        
        debug!("默克尔树构建完成，根哈希: {}", hex::encode(root.as_bytes()));
        Ok(root)
    }

    /// 生成交易的默克尔证明
    pub fn generate_proof(&self, transaction: &Transaction) -> Result<MerkleProof, LedgerError> {
        let tree = self.tree.as_ref()
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
        let index = index.ok_or_else(|| LedgerError::MerkleTreeError("交易未找到".to_string()))?;
    
        let proof = tree.gen_proof(index)
            .map_err(|e| LedgerError::MerkleTreeError(e.to_string()))?;
            
        let proof_hashes = proof.lemma().iter()
            .map(|h| Hash::from_slice(h))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| LedgerError::MerkleTreeError(e.to_string()))?;
    
        Ok(MerkleProof {
            proof_hashes,
            root_hash: Hash::from_slice(&tree.root())
                .map_err(|e| LedgerError::MerkleTreeError(e.to_string()))?
        })
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
        let mut current = Hash::from_slice(transaction_hash)
            .map_err(|e| e.to_string())?;
        
        for proof_hash in &self.proof_hashes {
            let combined = if current.as_ref() <= proof_hash.as_ref() {
                [current.as_ref(), proof_hash.as_ref()].concat()
            } else {
                [proof_hash.as_ref(), current.as_ref()].concat()
            };
            
            // 修复类型不匹配问题
        let hash_bytes = to_hash(combined.as_slice().to_vec())
            .map_err(|e| e.to_string())?;
        current = Hash::from_slice(hash_bytes.as_ref())
            .map_err(|e| e.to_string())?;
        }
        
        Ok(current == self.root_hash)
    }
}