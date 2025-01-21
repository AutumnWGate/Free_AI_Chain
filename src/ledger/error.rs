use thiserror::Error;
use std::sync::PoisonError;

#[derive(Debug, Error)]
pub enum LedgerError {
    #[error("数据库错误: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    
    #[error("序列化错误: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Merkle树错误: {0}")]
    MerkleTreeError(String),
    
    #[error("交易验证错误: {0}")]
    TransactionVerifyError(String),
    
    #[error("区块验证错误: {0}")]
    BlockVerifyError(String),

    #[error("钱包错误: {0}")]
    WalletError(String),

    #[error("状态锁定错误: {0}")]
    LockError(String),

    #[error("未知错误: {0}")]
    Unknown(String),

    #[error("初始化错误: {0}")]
    InitializationError(String),

    #[error("事件错误: {0}")]
    EventError(String),

    #[error("互斥锁毒化错误")]
    PoisonLockError,

    #[error("交易已确认: {0}")]
    TransactionAlreadyConfirmed(String),

    #[error("无效数据: {0}")]
    InvalidData(String),
}

/// 为 LedgerError 实现 From trait，用于处理 PoisonError 的转换，简化错误处理代码，使得可以直接使用 ? 运算符
impl<T> From<PoisonError<T>> for LedgerError {
    fn from(_: PoisonError<T>) -> Self {
        LedgerError::PoisonLockError
    }
}