use thiserror::Error;




/// 定义铸造模块的错误类型
#[derive(Error, Debug)]
pub enum MintingError {
    /// 权限被拒绝错误，例如没有管理员权限进行铸造
    #[error("权限被拒绝")]
    PermissionDenied,

    /// 无效的铸造数量错误，例如铸造数量为零或超过限制
    #[error("无效的铸造数量: {0}")]
    InvalidMintingAmount(String),

    /// 白名单错误
    #[error("白名单错误: {0}")]
    WhitelistError(#[from] WhitelistError),

    /// 数据库错误，当与数据库交互发生错误时返回
    #[error("数据库错误: {0}")]
    DatabaseError(#[from] sqlx::Error),

    /// 序列化/反序列化 错误
    #[error("序列化/反序列化 错误: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// 默克尔树错误，当与默克尔树交互发生错误时返回
    #[error("默克尔树错误: {0}")]
    MerkleTreeError(#[from] crate::types::merkletree::MerkleTreeError),

    /// 区块错误，当与区块交互发生错误时返回 (原 BlockchainError 更名为 BlockError)
    #[error("区块错误: {0}")]
    BlockError(#[from] crate::types::block::BlockError),

    /// 签名错误
    #[error("签名错误: {0}")]
    SignatureError(String),

    /// 铸造信息格式错误
    #[error("铸造信息格式错误: {0}")]
    InvalidMintingFormat(String),

    /// 十六进制解码错误
    #[error("十六进制解码错误: {0}")]
    HexDecodingError(#[from] hex::FromHexError),

    /// IO 错误
    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),

    /// 其他铸造模块未定义的错误
    #[error("其他铸造错误: {0}")]
    Other(String),
}

/// 白名单错误类型
#[derive(Error, Debug)]
pub enum WhitelistError {
    /// 地址不在白名单中
    #[error("地址不在白名单中: {0}")]
    AddressNotWhitelisted(String),

    /// 白名单配置错误
    #[error("白名单配置错误: {0}")]
    ConfigurationError(String),

    /// 其他白名单错误
    #[error("其他白名单错误: {0}")]
    Other(String),
}