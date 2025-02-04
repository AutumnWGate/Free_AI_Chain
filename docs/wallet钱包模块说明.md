# 钱包模块说明

## 开发要求
1. ios、android移动端采用本地密钥管理，使用flutter_secure_storage插件。
2. 加密存储的信息：
    1. 助记词
    2. 私钥
    3. 密码
    4. 节点连接信息 todo
    5. API 密钥/访问令牌 (Sensitive API Keys / Access Tokens) todo
    6. 其他敏感信息 todo
3. 不加密存储的信息：
    1. 地址
    2. 公钥
    3. nonce


## 注册要点：
1. 用户输入密码，生成地址和私钥，同时生成助记词。
2. 用户输入助记词，可以找回密码、地址和私钥。
3. 用户输入密码，可以找回助记词。
4. 密码复杂度要求：
    1、要求不低于8位。
    2、要求有最少一位是大写字母。

## 登录要点：
1. 当本地缓存用户信息时，用户输入密码，可以实现登录。
2. 当本地没有缓存用户信息时，用户输入助记词，可以实现登录且找回地址和私钥。

## 交易历史记录实现思路
让钱包模块作为用户接口层，而将具体的交易查询逻辑放在账本模块中。
- 钱包模块 (wallet::Wallet):
    提供面向用户的高层接口
    只关注与当前钱包地址相关的交易
    可以缓存最近的交易记录

- 账本模块 (ledger):
    提供底层的交易查询实现
    负责数据库访问和交易记录管理
    可以查询任意地址的交易



## 币种ID
| Coin type  | Path component (`coin_type'`) | Symbol  | Coin                              |
| ---------- | ----------------------------- | ------- | --------------------------------- |
| 1010101010 | 0xBC34EB12                    | FAIC    | Free AI Chain                     |



## 钱包模块结构
src/
└── wallet/
    ├── mod.rs          # 模块入口文件，导出子模块和公共接口
    ├── wallet.rs       # 钱包核心功能实现，如创建、导入、导出等
    ├── key_manager.rs  # 密钥管理，包括密钥生成、派生、存储等
    ├── address.rs      # 地址生成和验证，BIP 标准相关逻辑
    ├── mnemonic.rs     # 助记词生成和管理，BIP39 标准
    ├── error.rs        # 模块自定义错误类型
结构说明:
mod.rs: 模块入口文件，用于声明和导出 wallet 模块的子模块，以及定义模块的公共接口 (例如，可以定义一个 WalletManager 结构体或 trait，作为钱包模块的主要对外接口)。
wallet.rs: 包含 Wallet 结构体的定义，以及钱包的核心操作函数，例如 CreateWallet (创建新钱包), import_wallet_from_mnemonic (从助记词导入钱包), export_mnemonic (导出助记词) 、密码验证和密钥存储等。
key_manager.rs: 负责密钥的生成、派生和管理。主要功能包括：
   - 从助记词和密码生成 BIP39 种子
   - 使用 BIP32/BIP44 标准进行密钥派生
   - 管理扩展私钥和公钥
   - 提供签名和验证功能
address.rs: 处理钱包地址的生成和验证，实现BIP32、BIP44、BIP47、BIP84等地址生成标准。可以包含 generate_address (生成地址), validate_address (验证地址格式) 等函数。
mnemonic.rs: 专门处理助记词的生成和验证，实现 BIP39 标准。可以包含 generate_mnemonic (生成助记词), validate_mnemonic (验证助记词), mnemonic_to_seed (助记词转换为种子) 等函数。
error.rs: 定义 wallet 模块的自定义错误类型 WalletError，统一处理模块内部的错误，并方便上层模块进行错误处理。


## 钱包密钥管理的业务逻辑

### 核心数据结构
`KeyManager` 主要负责密钥的生成、派生和管理
```rust
pub struct KeyManager {
    master_key: XPrv,    // 扩展私钥，用于派生子私钥和签名
    pub xpub: XPub,      // 扩展公钥，用于派生子公钥和生成地址
    derivation_path: DerivationPath  // BIP44 派生路径
}
```


### 主要功能
1. 从助记词创建密钥管理器
    函数: `from_mnemonic(mnemonic: &str, password: &str) -> Result<Self, KeyManagerError>`
    业务流程：
        1. 验证密码复杂度
            1、要求不低于8位。
            2、要求有最少一位是大写字母。
        2. 解析并验证助记词
        3. 使用密码生成 BIP39 种子
        4. 从种子生成根私钥 (m)
        5. 创建标准 BIP44 派生路径：m/44'/FAIC_COIN_TYPE'/0'/0'/0
           - m: Master 根密钥
           - 44': BIP44 协议标识
           - FAIC_COIN_TYPE': 币种类型
           - 0': 账户索引
           - 0: 外部链/内部链
           - 0: 地址索引
        6. 派生主密钥对（子私钥和对应的公钥）
2. 创建标准 BIP44 派生路径
    函数: `create_bip44_path(account: u32, change: bool, address_index: u32) -> Result<DerivationPath, KeyManagerError>`
    业务流程：
       1. 验证参数范围
       2. 构建 BIP44 路径字符串：m/44'/FAIC_COIN_TYPE'/account'/change/address_index
       3. 验证派生路径的有效性
3. 验证派生路径
    函数: `validate_derivation_path(path: &DerivationPath) -> Result<(), KeyManagerError>`
    业务流程：
        1. 验证派生深度（最大5层）
        2. 验证各级索引：
           1. 第一级：必须是44'（硬化）
           2. 第二级：必须是 FAIC_COIN_TYPE'（硬化）
           3. 第三级：账户索引（硬化）
           4. 第四、五级：子密钥索引（非硬化）
4. 派生子密钥
    函数: `derive_key(&self, path: &str) -> Result<XPrv, KeyManagerError>`
    业务流程：
        1. 解析派生路径
        2. 验证路径有效性
        3. 从主密钥派生子密钥
5. 获取公钥
    函数: `get_public_key(&self, path: &str) -> Result<XPub, KeyManagerError>`
    业务流程：
        1. 派生指定路径的子密钥
        2. 获取对应的公钥
6. 签名消息
    函数: `sign_message(&self, message: &[u8]) -> Result<SignatureWrapper, KeyManagerError>`
    业务流程：
        1. 计算消息哈希（SHA256）
        2. 使用主私钥进行签名
        3. 生成可恢复的签名
        4. 返回签名包装器
7. 验证签名
    函数: `verify_signature(&self, message: &[u8], signature: &SignatureWrapper, public_key: &XPub) -> Result<bool, KeyManagerError>`
    业务流程：
        1. 计算消息哈希（SHA256）
        2. 使用公钥验证签名
        3. 返回验证结果
8. 常量定义
```rust
const FAIC_COIN_TYPE: u32 = 0xBC34EB12;  // FAIC 币种 ID
const MAX_DERIVATION_DEPTH: u8 = 5;       // BIP44 最大深度
const MAX_INDEX: u32 = 0x7fffffff;        // 最大索引值（非硬化）
const HARDENED_INDEX_START: u32 = 0x80000000;  // 硬化索引起始值
```
9. 错误处理
使用 KeyManagerError 枚举处理以下错误类型：
    1. 无效的助记词
    2. 密钥派生失败
    3. 签名错误
    4. 无效的派生路径
    5. 密码错误
所有错误都通过 Result 类型返回，确保安全的错误处理。