use std::sync::PoisonError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LedgerError {
    #[error("数据库错误: {0}")]
    DatabaseError(#[from] sqlx::Error),

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

    #[error("创建钱包错误: {0}")]
    CreateWalletError(String),

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

    #[error("反序列化错误: {0}")]
    DeserializationError(String),
    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),
    #[error("UTF8错误: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error("十六进制解码错误: {0}")]
    HexDecodingError(#[from] hex::FromHexError),

    #[error("金额错误: {0}")]
    AmountError(String),

    #[error("签名错误: {0}")]
    SignatureError(String),

    #[error("时间戳错误: {0}")]
    TimestampError(String),

    #[error("十六进制解码错误: {0}")]
    HexError(String),

    #[error("区块错误: {0}")]
    BlockError(String),
}

/// 为 LedgerError 实现 From trait，用于处理 PoisonError 的转换，简化错误处理代码，使得可以直接使用 ? 运算符
impl<T> From<PoisonError<T>> for LedgerError {
    fn from(_: PoisonError<T>) -> Self {
        LedgerError::PoisonLockError
    }
}

impl From<String> for LedgerError {
    fn from(err: String) -> LedgerError {
        LedgerError::WalletError(err)
    }
}
