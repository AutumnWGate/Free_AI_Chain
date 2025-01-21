use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use crate::types::amount::Amount;

// 交易类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Transfer, // 转账
    SmartContract, // 智能合约
    // 其他交易类型...
}

// 交易结构体
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    pub transaction_type: TransactionType,
    pub from: String,
    pub to: String,
    pub transfer_amount: Amount,
    pub nonce: u64,
    pub signature: Vec<u8>,
    pub timestamp: DateTime<Utc>,
    pub fee: Amount,
    #[serde(with = "hex")] // 使用 hex 库进行十六进制编码/解码
    pub transaction_hash: Vec<u8>,

}

impl Default for Transaction {
    fn default() -> Self {
        Transaction {
            transaction_type: TransactionType::Transfer,
            from: String::new(),
            to: String::new(),
            transfer_amount: Amount::default(),
            nonce: 0,
            signature: Vec::new(),
            timestamp: Utc::now(),
            fee: Amount::default(),
            transaction_hash: Vec::new(),
        }
    }
}

impl AsRef<[u8]> for Transaction {
    fn as_ref(&self) -> &[u8] {
        let serialized = serde_json::to_vec(self).unwrap();
        let boxed_slice = serialized.into_boxed_slice();
        Box::leak(boxed_slice)
    }
}

impl PartialOrd for Transaction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Transaction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.transaction_hash.cmp(&other.transaction_hash)
    }
}

impl Transaction {
    // 计算并设置交易哈希
    pub fn calculate_and_set_hash(&mut self) {
        self.transaction_hash = calculate_transaction_hash(self);
    }
}

// 计算交易哈希的函数
pub fn calculate_transaction_hash(transaction: &Transaction) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(serde_json::to_vec(&transaction.transaction_type).unwrap());
    hasher.update(&transaction.from);
    hasher.update(&transaction.to);
    hasher.update(transaction.transfer_amount.to_bytes_be());
    hasher.update(transaction.nonce.to_be_bytes());
    hasher.update(&transaction.signature);
    hasher.update(transaction.timestamp.to_rfc3339().as_bytes());
    hasher.update(transaction.fee.to_bytes_be());
    hasher.finalize().to_vec()
}

// 交易池
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionPool {
    pub transactions: Vec<Transaction>,
}

impl TransactionPool {
    pub fn new() -> Self {
        TransactionPool {
            transactions: Vec::new(),
        }
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }

    pub fn remove_transaction(&mut self, transaction_hash: &Vec<u8>) {
        self.transactions.retain(|transaction| &transaction.transaction_hash != transaction_hash);
    }

    pub fn get_transactions_for_block(&self, max_transactions: usize) -> Vec<Transaction> {
        // TODO: 根据交易费用或其他策略选择交易
        self.transactions.iter().take(max_transactions).cloned().collect()
    }
}

