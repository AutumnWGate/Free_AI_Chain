use thiserror::Error;

/// 钱包模块错误类型
#[derive(Debug, Error)]
pub enum WalletError {
    /// 密钥管理相关错误
    #[error("密钥管理错误: {0}")]
    KeyManager(#[from] KeyManagerError),
    
    #[error("签名错误: {0}")]
    SignatureError(String),

    /// 地址相关错误
    #[error("地址错误: {0}")]
    AddressError(String),

    /// 助记词相关错误
    #[error("助记词错误: {0}")]
    Mnemonic(String),

    /// 数据库操作错误
    #[error("数据库错误: {0}")]
    Database(String),

    /// 支付码相关错误
    #[error("支付码错误: {0}")]
    PaymentCode(String),

    /// SegWit 地址错误
    #[error("SegWit 地址错误: {0}")]
    SegwitAddressError(String),

    /// 无效的密码
    #[error("无效的密码: {0}")]
    InvalidPassword(String),

    /// 助记词生成错误
    #[error("助记词生成错误: {0}")]
    MnemonicGeneration(String),

    /// 没有助记词
    #[error("没有助记词")]
    NoMnemonic,

    /// 加密错误
    #[error("加密错误: {0}")]
    EncryptionError(String),

    /// 解密错误
    #[error("解密错误: {0}")]
    DecryptionError(String),

}

/// 密钥管理相关错误
#[derive(Debug, Error)]
pub enum KeyManagerError {
    /// 无效的助记词
    #[error("无效的助记词: {0}")]
    InvalidMnemonic(String),

    /// 密钥派生失败
    #[error("密钥派生失败: {0}")]
    DerivationError(String),

    /// 签名错误
    #[error("签名错误: {0}")]
    SignatureError(String),

    /// 无效的派生路径
    #[error("无效的派生路径: {0}")]
    InvalidPath(String),

    /// 密码错误
    #[error("密码错误: {0}")]
    InvalidPassword(String),

    /// 路径深度错误
    #[error("路径深度错误: {0}")]
    PathDepthError(String),

    /// 路径组件错误
    #[error("路径组件错误: {0}")]
    PathComponentError(String),

    /// 种子生成错误
    #[error("种子生成错误: {0}")]
    SeedGenerationError(String),
    
    /// 没有密钥管理器
    #[error("没有密钥管理器")]
    NoKeyManager,

}