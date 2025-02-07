use crate::crypto::hash::{sha256_concat, Hash};
use crate::crypto::signature::SignatureWrapper;
use crate::types::amount::Amount;
use chrono::{DateTime, Utc};
use log::debug;
use merkletree::merkle::Element;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

// 使用静态缓冲区来存储序列化结果
thread_local! {
    static BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::new());
}

#[derive(Debug, thiserror::Error)]
pub enum TransactionError {
    #[error("序列化错误: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("签名错误: {0}")]
    SignatureError(String),
    #[error("哈希错误: {0}")]
    HashError(#[from] crate::crypto::hash::HashError),
}

// 交易类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Transfer,      // 转账
    SmartContract, // 智能合约
    Dapp,          // dapp
    Mint,   // 铸造
                   // 其他交易类型...
}

// 交易状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    AwaitVerify, // 等待验证
    Confirmed,   // 已确认
    Pending,     // 挂起等待人工处理
    Processing,  // 处理中
    Rejected,    // 已拒绝
    Expired,     // 已过期
    Unknown,     // 未知
}

// 交易信息结构体
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionDetail {
    pub transaction_type: TransactionType,
    pub from: String,
    pub to: String,
    pub transfer_amount: Amount,
    pub locked: bool,
    pub unlocked_time: DateTime<Utc>,
    pub nonce: i64,
    pub timestamp: DateTime<Utc>,
    pub fee: Amount,
    pub transaction_hash: Hash,
    pub transaction_status: String,
    pub initiator_signature: SignatureWrapper,
}

impl TransactionDetail {
    /// 创建新交易
    pub fn new(
        transaction_type: TransactionType,
        from: String,
        to: String,
        transfer_amount: Amount,
        locked: bool,
        unlocked_time: DateTime<Utc>,
        nonce: i64,
        fee: Amount,
        initiator_signature: SignatureWrapper,
    ) -> Self {
        let mut transaction = TransactionDetail {
            transaction_type,
            from,
            to,
            transfer_amount,
            locked,
            unlocked_time,
            nonce,
            initiator_signature,
            timestamp: Utc::now(),
            fee,
            transaction_hash: Hash::new(),
            transaction_status: TransactionStatus::AwaitVerify.to_string(),
        };

        // 计算交易哈希
        if let Ok(hash) = calculate_transaction_hash(&transaction) {
            transaction.transaction_hash = hash;
        }

        transaction
    }

    /// 将交易序列化为字节数组
    pub fn to_bytes(&self) -> Result<Vec<u8>, TransactionError> {
        serde_json::to_vec(self).map_err(TransactionError::SerializationError)
    }

    /// 从字节数组反序列化为交易
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, TransactionError> {
        serde_json::from_slice(bytes).map_err(TransactionError::SerializationError)
    }

    /// 计算并设置交易哈希
    pub fn calculate_and_set_hash(&mut self) -> Result<(), TransactionError> {
        debug!("正在计算交易哈希");
        self.transaction_hash = calculate_transaction_hash(self)?;
        debug!("交易哈希计算完成: {:?}", self.transaction_hash);
        Ok(())
    }
}

impl Default for TransactionDetail {
    fn default() -> Self {
        TransactionDetail {
            transaction_type: TransactionType::Transfer,
            from: String::new(),
            to: String::new(),
            transfer_amount: Amount::default(),
            nonce: 0,
            initiator_signature: SignatureWrapper::from_bytes(&[0u8; 65])
                .expect("Default signature should be valid"),
            timestamp: Utc::now(),
            fee: Amount::default(),
            transaction_hash: Hash::new(),
            transaction_status: TransactionStatus::AwaitVerify.to_string(),
            locked: false,
            unlocked_time: Utc::now(),
        }
    }
}

impl std::fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionStatus::AwaitVerify => write!(f, "AwaitVerify"),
            TransactionStatus::Confirmed => write!(f, "Confirmed"),
            TransactionStatus::Pending => write!(f, "Pending"),
            TransactionStatus::Processing => write!(f, "Processing"),
            TransactionStatus::Rejected => write!(f, "Rejected"),
            TransactionStatus::Expired => write!(f, "Expired"),
            TransactionStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

impl PartialOrd for TransactionDetail {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TransactionDetail {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.transaction_hash.cmp(&other.transaction_hash)
    }
}

// 实现 AsRef<[u8]> trait
impl AsRef<[u8]> for TransactionDetail {
    fn as_ref(&self) -> &[u8] {
        BUFFER.with(|buffer| {
            let mut buf = buffer.borrow_mut();
            *buf = serde_json::to_vec(self).unwrap_or_default();
            // 安全：buf 的生命周期与 BUFFER 绑定
            unsafe { std::mem::transmute(buf.as_slice()) }
        })
    }
}

impl Element for TransactionDetail {
    fn byte_len() -> usize {
        let sample = TransactionDetail::default();
        serde_json::to_vec(&sample).unwrap_or_default().len()
    }

    fn from_slice(bytes: &[u8]) -> Self {
        serde_json::from_slice(bytes).unwrap_or_default()
    }

    fn copy_to_slice(&self, bytes: &mut [u8]) {
        if let Ok(serialized) = serde_json::to_vec(self) {
            let len = serialized.len().min(bytes.len());
            if len > 0 {
                bytes[..len].copy_from_slice(&serialized[..len]);
                if bytes.len() > len {
                    bytes[len..].fill(0);
                }
            }
        }
    }
}

// 计算交易哈希的函数
pub fn calculate_transaction_hash(
    transaction: &TransactionDetail,
) -> Result<Hash, TransactionError> {
    let type_bytes = serde_json::to_vec(&transaction.transaction_type)
        .map_err(TransactionError::SerializationError)?;

    sha256_concat(&[
        &type_bytes,
        transaction.from.as_bytes(),
        transaction.to.as_bytes(),
        &transaction.transfer_amount.to_bytes_be(),
        &transaction.nonce.to_be_bytes(),
        transaction.timestamp.to_rfc3339().as_bytes(),
        &transaction.fee.to_bytes_be(),
        &transaction.transaction_status.as_bytes(),
        &[transaction.locked as u8],
        &transaction.unlocked_time.to_rfc3339().as_bytes(),
    ])
    .map_err(TransactionError::HashError)
}

// 交易池
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionPool {
    pub transactions: Vec<TransactionDetail>,
}

impl TransactionPool {
    pub fn new() -> Self {
        TransactionPool {
            transactions: Vec::new(),
        }
    }

    pub fn add_transaction(&mut self, transaction: TransactionDetail) {
        self.transactions.push(transaction);
    }

    pub fn remove_transaction(&mut self, transaction_hash: &Hash) {
        self.transactions
            .retain(|transaction| &transaction.transaction_hash != transaction_hash);
    }

    pub fn get_transactions_for_block(&self, max_transactions: usize) -> Vec<TransactionDetail> {
        // TODO: 根据交易费用或其他策略选择交易
        self.transactions
            .iter()
            .take(max_transactions)
            .cloned()
            .collect()
    }

    // 添加按费用排序的方法
    pub fn sort_by_fee(&mut self) {
        self.transactions.sort_by(|a, b| b.fee.cmp(&a.fee));
    }

    // 添加验证交易是否存在的方法
    pub fn contains(&self, transaction_hash: &Hash) -> bool {
        self.transactions
            .iter()
            .any(|tx| &tx.transaction_hash == transaction_hash)
    }

    // 添加获取交易池大小的方法
    pub fn size(&self) -> usize {
        self.transactions.len()
    }

    // 添加清理过期交易的方法
    pub fn remove_expired_transactions(&mut self, expiration: DateTime<Utc>) {
        self.transactions.retain(|tx| tx.timestamp > expiration);
    }
}
