pub mod address;
pub mod hash;
pub mod signature;

pub type CryptoResult<T> = Result<T, CryptoError>;

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Invalid hash")]
    InvalidHash,
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Hex decode error: {0}")]
    HexError(#[from] hex::FromHexError),
    // ...
}