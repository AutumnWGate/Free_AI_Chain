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

    #[error("交易未找到")]
    TransactionNotFound,
    #[error("无效的交易")]
    InvalidTransaction,
    #[error("无效的区块")]
    InvalidBlock,
    #[error("无效的账户")]
    InvalidAccount,
    #[error("余额不足")]
    InsufficientBalance,
    #[error("无效的签名")]
    InvalidSignature,
    #[error("无效的公钥")]
    InvalidPublicKey,
    #[error("无效的哈希")]
    InvalidHash,
    #[error("无效的难度")]
    InvalidDifficulty,
    #[error("无效的时间戳")]
    InvalidTimestamp,
    #[error("无效的版本")]
    InvalidVersion,
    #[error("无效的交易输入")]
    InvalidTransactionInput,
    #[error("无效的交易输出")]
    InvalidTransactionOutput,
    #[error("无效的交易费用")]
    InvalidTransactionFee,
    #[error("无效的交易锁定时间")]
    InvalidTransactionLockTime,
    #[error("无效的交易大小")]
    InvalidTransactionSize,
    #[error("无效的交易数据")]
    InvalidTransactionData,
    #[error("无效的区块头")]
    InvalidBlockHeader,
    #[error("无效的区块体")]
    InvalidBlockBody,
    #[error("无效的区块哈希")]
    InvalidBlockHash,
    #[error("无效的区块高度")]
    InvalidBlockHeight,
    #[error("无效的区块时间戳")]
    InvalidBlockTimestamp,
    #[error("无效的区块默克尔根")]
    InvalidBlockMerkleRoot,
    #[error("无效的区块验证者")]
    InvalidBlockValidator,
    #[error("无效的区块签名")]
    InvalidBlockSignature,
    #[error("无效的区块数量")]
    InvalidBlockNumber,
    #[error("无效的前一个区块哈希")]
    InvalidPreviousBlockHash,
    #[error("无效的账户地址")]
    InvalidAccountAddress,
    #[error("无效的账户余额")]
    InvalidAccountBalance,
    #[error("无效的账户随机数")]
    InvalidAccountNonce,
    #[error("无效的账户代码")]
    InvalidAccountCode,
    #[error("无效的账户存储")]
    InvalidAccountStorage,
    #[error("无效的账户密钥")]
    InvalidAccountKey,
    #[error("无效的账户数据")]
    InvalidAccountData,
    #[error("无效的账户版本")]
    InvalidAccountVersion,
    #[error("无效的账户类型")]
    InvalidAccountType,
    #[error("无效的账户权限")]
    InvalidAccountPermission,
    #[error("无效的账户状态")]
    InvalidAccountStatus,
    #[error("无效的账户创建时间")]
    InvalidAccountCreateTime,
    #[error("无效的账户更新时间")]
    InvalidAccountUpdateTime,
    #[error("无效的账户哈希")]
    InvalidAccountHash,
    #[error("无效的账户签名")]
    InvalidAccountSignature,
    #[error("无效的账户公钥")]
    InvalidAccountPublicKey,
    #[error("无效的账户私钥")]
    InvalidAccountPrivateKey,
    #[error("无效的账户助记词")]
    InvalidAccountMnemonic,
    #[error("无效的账户路径")]
    InvalidAccountPath,
    #[error("无效的账户索引")]
    InvalidAccountIndex,
    #[error("无效的账户链")]
    InvalidAccountChain,
    #[error("无效的账户网络")]
    InvalidAccountNetwork,
    #[error("无效的账户币种")]
    InvalidAccountCoin,
    #[error("无效的账户单位")]
    InvalidAccountUnit,
    #[error("无效的账户精度")]
    InvalidAccountDecimals,
    #[error("无效的账户符号")]
    InvalidAccountSymbol,
    #[error("无效的账户名称")]
    InvalidAccountName,
    #[error("无效的账户描述")]
    InvalidAccountDescription,
    #[error("无效的账户图标")]
    InvalidAccountIcon,
    #[error("无效的账户网址")]
    InvalidAccountUrl,
    #[error("无效的账户邮箱")]
    InvalidAccountEmail,
    #[error("无效的账户电话")]
    InvalidAccountPhone,
    #[error("无效的账户地址簿")]
    InvalidAccountAddressBook,
    #[error("无效的账户联系人")]
    InvalidAccountContact,
    #[error("无效的账户消息")]
    InvalidAccountMessage,
    #[error("无效的账户通知")]
    InvalidAccountNotification,
    #[error("无效的账户设置")]
    InvalidAccountSetting,
    #[error("无效的账户安全")]
    InvalidAccountSecurity,
    #[error("无效的账户隐私")]
    InvalidAccountPrivacy,
    #[error("无效的账户合规")]
    InvalidAccountCompliance,
    #[error("无效的账户审计")]
    InvalidAccountAudit,
    #[error("无效的账户报告")]
    InvalidAccountReport,
    #[error("无效的账户分析")]
    InvalidAccountAnalytics,
    #[error("无效的账户指标")]
    InvalidAccountMetrics,
    #[error("无效的账户日志")]
    InvalidAccountLog,
    #[error("无效的账户事件")]
    InvalidAccountEvent,
    #[error("无效的账户操作")]
    InvalidAccountOperation,
    #[error("无效的账户记录")]
    InvalidAccountRecord,
    #[error("无效的账户历史")]
    InvalidAccountHistory,
    #[error("无效的账户快照")]
    InvalidAccountSnapshot,
    #[error("无效的账户备份")]
    InvalidAccountBackup,
    #[error("无效的账户恢复")]
    InvalidAccountRestore,
    #[error("无效的账户迁移")]
    InvalidAccountMigrate,
    #[error("无效的账户导入")]
    InvalidAccountImport,
    #[error("无效的账户导出")]
    InvalidAccountExport,
    #[error("无效的账户同步")]
    InvalidAccountSync,
    #[error("无效的账户异步")]
    InvalidAccountAsync,
    #[error("无效的账户并发")]
    InvalidAccountConcurrency,
    #[error("无效的账户并行")]
    InvalidAccountParallelism,
    #[error("无效的账户分布式")]
    InvalidAccountDistribution,
    #[error("无效的账户去中心化")]
    InvalidAccountDecentralization,
    #[error("无效的账户共识")]
    InvalidAccountConsensus,
    #[error("无效的账户治理")]
    InvalidAccountGovernance,
    #[error("无效的账户智能合约")]
    InvalidAccountSmartContract,
    #[error("无效的账户虚拟机")]
    InvalidAccountVirtualMachine,
    #[error("无效的账户字节码")]
    InvalidAccountBytecode,
    #[error("无效的账户接口")]
    InvalidAccountInterface,
    #[error("无效的账户函数")]
    InvalidAccountFunction,


    #[error("无效的账户状态")]
    InvalidAccountState,
    #[error("无效的账户交易")]
    InvalidAccountTransaction,
    #[error("无效的账户收据")]
    InvalidAccountReceipt,

    #[error("无效的账户调用")]
    InvalidAccountCall,


    #[error("无效的账户证明")]
    InvalidAccountProof,
    #[error("无效的账户验证")]
    InvalidAccountVerification,
    #[error("无效的账户确认")]
    InvalidAccountConfirmation,

    #[error("无效的账户分片")]
    InvalidAccountSharding,
    #[error("无效的账户跨链")]
    InvalidAccountCrossChain,
    #[error("无效的账户侧链")]
    InvalidAccountSideChain,
    #[error("无效的账户状态通道")]
    InvalidAccountStateChannel,
    #[error("无效的账户支付通道")]
    InvalidAccountPaymentChannel,
    #[error("无效的账户原子交换")]
    InvalidAccountAtomicSwap,
    #[error("无效的账户多重签名")]
    InvalidAccountMultiSig,
    #[error("无效的账户门限签名")]
    InvalidAccountThresholdSig,
    #[error("无效的账户聚合签名")]
    InvalidAccountAggregateSig,
    #[error("无效的账户环签名")]
    InvalidAccountRingSig,
    #[error("无效的账户零知识证明")]
    InvalidAccountZkProof,
    #[error("无效的账户默克尔证明")]
    InvalidAccountMerkleProof,
    #[error("无效的账户布隆过滤器")]
    InvalidAccountBloomFilter,
    #[error("无效的账户加密")]
    InvalidAccountEncryption,
    #[error("无效的账户解密")]
    InvalidAccountDecryption,

    #[error("无效的账户编码")]
    InvalidAccountEncoding,
    #[error("无效的账户解码")]
    InvalidAccountDecoding,
    #[error("无效的账户序列化")]
    InvalidAccountSerialization,
    #[error("无效的账户反序列化")]
    InvalidAccountDeserialization,

    #[error("反序列化错误: {0}")]
    DeserializationError(String),
    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),
    #[error("UTF8错误: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),



    #[error("十六进制解码错误: {0}")]
    HexDecodingError(#[from] hex::FromHexError),

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

impl From<sqlx::Error> for LedgerError {
    fn from(err: sqlx::Error) -> Self {
        LedgerError::DatabaseError(err.to_string())
    }
}