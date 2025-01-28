use crate::network::config::NetworkConfigError;
use libp2p::multiaddr::Error as MultiaddrError;
use libp2p::noise::Error as NoiseError;
use libp2p::swarm::DialError;
use std::convert::Infallible;

/// 定义整个 faic_core 项目的通用错误类型
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Network config error: {0}")]
    NetworkConfig(#[from] NetworkConfigError),

    #[error("Network error: {0}")]
    Network(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(String),

    #[error("Invalid address")]
    InvalidAddress,

    #[error("Insufficient balance")]
    InsufficientBalance,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Invalid timestamp")]
    InvalidTimestamp,

    #[error("Invalid hash")]
    InvalidHash,

    #[error("Invalid nonce")]
    InvalidNonce,

    #[error("Invalid amount")]
    InvalidAmount,

    #[error("Failed to add transaction to pool")]
    AddTransactionToPoolFailed,

    #[error("Failed to get node info")]
    GetNodeInfoFailed,

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Not found")]
    NotFound,

    #[error("Invalid Merkle proof")]
    InvalidMerkleProof,

    #[error("Merkle tree error")]
    MerkleTreeError,
}

// 可以在这里添加其他错误类型的转换，例如：
// impl From<WalletError> for Error {
//     fn from(err: WalletError) -> Self {
//         Error::Wallet(err)
//     }
// }
//
// impl From<TransactionError> for Error {
//     fn from(err: TransactionError) -> Self {
//         Error::Transaction(err)
//     }
// }

// 示例：添加一个网络错误类型的转换
impl From<libp2p::TransportError<std::io::Error>> for Error {
    fn from(err: libp2p::TransportError<std::io::Error>) -> Self {
        Error::Network(err.to_string())
    }
}

impl From<DialError> for Error {
    fn from(error: DialError) -> Self {
        Error::Network(error.to_string())
    }
}

impl From<MultiaddrError> for Error {
    fn from(error: MultiaddrError) -> Self {
        Error::Network(error.to_string())
    }
}

impl From<libp2p::swarm::ListenError> for Error {
    fn from(error: libp2p::swarm::ListenError) -> Self {
        Error::Network(error.to_string())
    }
}

impl From<NoiseError> for Error {
    fn from(error: NoiseError) -> Self {
        Error::Network(error.to_string())
    }
}

// 为 Error 实现 From<Infallible>
impl From<Infallible> for Error {
    fn from(_: Infallible) -> Self {
        // Infallible 永远不会发生，所以这个实现实际上永远不会被调用
        unreachable!("Infallible error should never occur")
    }
}
