# `ledger` 账本模块

`ledger` 模块是 FAIC 区块链的核心组件之一，负责维护整个区块链的账本状态。它像一个超级账本，记录了所有账户的余额、每一笔交易的详细信息以及所有区块的信息。

MVP001版本技术栈要求：
*   Rust
*   sqlx
*   Merkletree
*   serde
*   serde_json

主要功能：
*   **钱包管理**`wallet_management`:
    *   钱包记录`wallet_record`: 记录和查询每个账户的地址`address`、余额`balance`、交易序号`nonce`、助记词`mnemonic`、私钥`private_key`、公钥`public_key`、钱包创建时间`create_time`。
    *   钱包操作`wallet_action`: 创建`create_wallet`、删除`delete_wallet`、查询余额`get_balance`、查询交易历史`get_transaction_history`。
*   **交易管理**`transaction_management`:
    *   交易记录`transaction_info`: 记录和查询每一笔交易的详细信息，包括交易类型`transaction_type`、发送方`from`、接收方`to`、金额`transfer_amount`、交易序号`nonce`、签名`signature`、时间戳`timestamp`、费用`fee`和哈希值`hash`。交易记录需要确保不可篡改，通常通过哈希和区块链接来保证。需要确保交易记录的持久化存储。
    *   交易操作`transaction_action`: 执行转账操作。
    *   交易池`transaction_pool`: 交易池是未确认的交易的集合，用于存储等待确认的交易。
    *   交易验证`transaction_verify`: 验证每一笔交易的合法性，确保交易的有效性。
*   **区块管理**`blockchain区块链模块`:
    *   区块记录`block_header`: 存储元数据，用于维护区块链的结构、安全性和基本属性。记录和查询每一个区块的信息，包括区块头信息（父区块哈希`parent_hash`、高度`height`、时间戳`timestamp`、Merkle 根`merkle_root`、验证者`validator`、签名`signature`、区块哈希`block_hash`）。
    *   区块体`block_body`: 存储区块中的所有交易数据`transaction_list`,智能合约相关的执行结果和日志 (Smart Contract Execution Results and Logs)
    *   区块创建`block_create`: 根据POS共识机制，创建新的区块。
    *   区块验证`block_verify`: 验证区块的合法性。
*   **Merkle 树**`merkle_tree`: 使用 `merkletree` 库来高效地验证交易是否存在于某个区块中，并生成相应的证明。在交易管理和区块管理中都会用到 Merkle 树。
    *   使用 `merkle_tree::new` 或 `merkle_tree::from_data` 构建 Merkle 树。
    *   使用 `merkle_tree::gen_proof` 生成 Merkle 证明。
    *   使用 `proof::validate` 验证 Merkle 证明。
    *   数据类型需要实现 `element` trait，以支持序列化和反序列化。

*   **ledger账本模块结构**:  
        ledger账本模块 --> wallet_management钱包管理
        ledger账本模块 --> blockchain区块链模块
        ledger账本模块 --> merkle_tree默克尔树
        ledger账本模块 --> transaction_management交易管理
        wallet_management钱包管理 --> wallet_record钱包记录
        wallet_management钱包管理 --> wallet_action钱包操作
        transaction_management交易管理 --> transaction_info交易记录
        transaction_management交易管理 --> transaction_action交易操作
        transaction_management交易管理 --> transaction_pool交易池
        transaction_management交易管理 --> transaction_verify交易验证
        blockchain区块链模块 --> block_header区块头
        blockchain区块链模块 --> block_body区块体
        blockchain区块链模块 --> block_create区块创建
        blockchain区块链模块 --> block_verify区块验证

## 数据结构

*   **钱包 `wallet`**:
    *   `address`: 字符串类型，表示钱包的地址。
    *   `available_balance`:  使用Amount类型，表示钱包的可用余额。
    *   `locked_balance`: 使用Amount类型，表示钱包的锁定余额。
    *   `nonce`: i64，表示钱包的动作序号，用于防止重放攻击。
  
*   **钱包操作 `wallet_action`**:
    *   `create_wallet`: 创建钱包。
    *   `delete_wallet`: 删除钱包。
    *   `get_balance`: 查询钱包余额。
    *   `get_transaction_history`: 查询钱包交易历史。

*   **签名类型 `signature_type`**:
    *   `initiator_signature`: 发起者签名，string类型, secp256k1签名,方法实现文件：src/crypto/signature.rs。

*   **交易类型 `transaction_type`**:
    *   `transfer`: 表示转账交易。
    *   `smart_contract`: 表示智能合约相关的交易。
    *   `dapp`: 表示dapp相关的交易。
    *   `early_donate`: 表示早期捐赠相关的交易，该类型款项用于FAIC早期开发，由开发者手动接收捐赠与分发代币。
    *   未来可以扩展其他类型的交易。

*   **交易信息结构体 `transaction_detail`**:
    *   `transaction_type`:  交易类型 (`transaction_type`)，表示交易的具体类型。
    *   `from`: 字符串类型，表示交易的发送方地址。
    *   `to`: 字符串类型，表示交易的接收方地址。
    *   `transfer_amount`: 使用Amount类型，表示交易的转账金额。
    *   `locked`：true or false，表示该笔交易是否被锁定。
    *   `nonce`: i64，表示交易的序号，用于防止重放攻击。
    *   `initiator_signature`: 发起者签名，string类型, secp256k1签名,方法实现文件：src/crypto/signature.rs。
    *   `timestamp`: 时间戳，表示交易发生的时间。
    *   `fee`: 使用Amount类型，表示交易的手续费。
    *   `transaction_hash`: 字节数组，表示交易的哈希值，用于唯一标识一笔交易。该字段在序列化和反序列化时使用十六进制编码。

*   **区块 `block`**:
    *   `block_header`: 区块头 (`block_header`)，包含区块的元数据信息。
    *   `block_body`: 区块体(`block_body`)，包含该区块打包的所有数据`data`，如：该区块的所有交易`transaction_payloads_for_block`、智能合约执行结果`smart_contract_payloads_for_block`。

*   **区块头 `block_header`**:
    *   `parent_hash`: 字节数组，表示父区块的哈希值。
    *   `height`: 64 位无符号整数，表示区块的高度。
    *   `timestamp`: 时间戳，表示区块生成的时间。
    *   `merkle_root`: 字节数组，表示区块中所有交易的 Merkle 树根哈希值。该字段在序列化和反序列化时使用十六进制编码。
    *   `validator`: 实现了 `Validator` 特征的类型，用于标识区块的验证者**用于标识验证区块的验证者身份**。
    *   `block_header_signature`: 字节数组，表示**区块头签名，由验证者对区块头进行签名生成，用于提供区块合法性和完整性的密码学证明**。
    *   `block_hash`: 字节数组，表示区块的哈希值，用于唯一标识一个区块。该字段在序列化和反序列化时使用十六进制编码。
*   **说明：**  `validator` 字段和 `block_header_signature` 字段共同协作，确保区块的安全性。 `validator` 标识区块的验证者，`block_header_signature` 提供验证者对区块的密码学签名。
  
*   **区块体 `block_body`**:
    *   `transaction_payloads_for_block`: 字节数组，表示区块中所有交易的数据。
    *   `smart_contract_payloads_for_block`: 字节数组，表示区块中所有智能合约的数据。
    *   `version`: 字节，表示区块的版本信息。

*   **交易哈希计算**:
    *   交易的哈希值是通过对交易的各个字段进行 SHA256 哈希计算得到的。
    *   具体来说，将交易类型、发送方地址、接收方地址、转账金额、交易序号、签名、时间戳和手续费依次拼接起来，然后进行 SHA256 哈希计算，得到的结果即为交易的哈希值。

*   **区块哈希计算**:
    *   区块的哈希值是通过对区块头的各个字段进行 SHA256 哈希计算得到的。
    *   具体来说，将父区块哈希、区块高度、时间戳、Merkle 根、验证者地址和签名依次拼接起来，然后进行 SHA256 哈希计算，得到的结果即为区块的哈希值。

*   **交易池 `transaction_pool`**:
    *   `await_confirm_transaction_list`: 等待确认的交易列表，用于存储待确认的交易 (`transaction_detail`)。
    *   `add_transaction()`:  添加一笔交易到交易池中。
    *   `remove_transaction()`: 根据交易哈希从交易池中移除一笔交易。
    *   `get_transaction_for_block()`:  从交易池中选择一定数量的交易，用于打包到新的区块中。选择交易的策略可以根据交易费用或其他因素来决定。

*   **Merkle 树相关说明**:
    *   交易 (`transaction_detail`) 和区块 (`block_header`)、区块体 (`block_body`) 都需要实现 `Element` trait，以便能够被 `merkletree` 库用于构建 Merkle 树。
    *   `Element` trait 要求实现以下方法：
        - `byte_len()` - 返回序列化后的字节长度
        - `from_slice(bytes: &[u8]) -> Self` - 从字节切片反序列化
        - `copy_to_slice(&self, bytes: &mut [u8])` - 序列化到字节切片
        - `hash(&self) -> Vec<u8>` - 返回数据的哈希值
    *   我们使用 `serde_json` 进行序列化和反序列化，使用 `SHA-256` 进行哈希计算。

## 数据库

我们使用 `sqlx` 作为数据库。以下是数据库表结构设计：

*   **表：`wallets` (钱包表)**
    *   `address` TEXT PRIMARY KEY (账户地址，主键)
    *   `available_balance` Amount.to_string() (可用余额)
    *   `locked_balance` Amount.to_string() (锁定余额)
    *   `nonce` INTEGER (动作序号)，触发交易、智能合约等导致账本数据库和区块变化的动作，都会增加nonce。

*   **表：`transaction` (交易表)**
    *   `transaction_hash` TEXT PRIMARY KEY (交易哈希，主键)
    *   `transaction_type` TEXT (交易类型)
    *   `from_address` TEXT (发送方地址)
    *   `to_address` TEXT (接收方地址)
    *   `transfer_amount` Amount.to_string() (转账金额)
    *   `nonce` INTEGER (交易序号)
    *   `signature` TEXT (签名)
    *   `timestamp` INTEGER (时间戳)
    *   `fee` Amount.to_string() (手续费)
    *   `block_hash` TEXT (所属区块哈希)
    *   `block_height` INTEGER (所属区块高度)
    *   FOREIGN KEY (`block_hash`) REFERENCES `blocks`(`block_hash`)

*   **表：`blocks` (区块表)**
    *   `block_hash` TEXT PRIMARY KEY (区块哈希，主键)
    *   `parent_hash` TEXT (父区块哈希)
    *   `block_height` INTEGER (区块高度)
    *   `timestamp` INTEGER (时间戳)
    *   `merkle_root` TEXT (Merkle 根)
    *   `validator` TEXT (验证者)
    *   `signature` TEXT (签名)

*   **表：`transaction_pool` (交易池表)**
    *   `transaction_hash` TEXT PRIMARY KEY (交易哈希，主键)
    *   `transaction_type` TEXT (交易类型)
    *   `from_address` TEXT (发送方地址)
    *   `to_address` TEXT (接收方地址)
    *   `transfer_amount` Amount.to_string() (转账金额)
    *   `nonce` INTEGER (交易序号)
    *   `signature` TEXT (签名)
    *   `timestamp` INTEGER (时间戳)
    *   `fee` Amount.to_string() (手续费)

**索引：**

*   在 `transactions` 表的 `from_address`、`to_address` 和 `block_hash` 列上创建索引，以加速交易查询。
*   在 `blocks` 表的 `parent_hash` 列上创建索引，以加速区块查询。

**事务：**

*   `sqlx` 支持事务，用于确保数据的一致性和完整性。
*   在执行创建钱包、转账、创建区块等操作时，需要使用事务来保证操作的原子性。

## 持久化

*   **存储方式：** 使用 `sqlx` 数据库文件来存储账本数据。
*   **数据备份：**
    *   定期备份数据库文件。
    *   可以使用 `sqlx` 提供的备份 API 来创建数据库的备份。
*   **数据恢复：**
    *   从备份文件中恢复数据库。
    *   可以使用 `sqlx` 提供的恢复 API 来从备份中恢复数据。
*   **数据目录：**
    *   数据库文件存储在用户可配置的数据目录中。
    *   默认情况下，数据目录位于用户的主目录下的 `.faic` 目录中。
*   

## 转账核心规则
  1、验证转账动作是否有效
    1、余额是否足够
    2、地址是否正确
    3、转账是否重复
  2、验证成功后，锁定转账金额，创建交易记录，存入交易池，等待确认
  3、确认后，将交易记录存入数据库和区块中
  4、更新地址余额
  5、通知客户端

## Nonce的定义和作用：
用户涉及的账本数据库和区块变化等写入的动作，都会增加nonce。


1、Nonce是每个账户的动作序号；
2、从0开始，严格递增；
3、用于防止交易重放攻击；
4、确保交易按顺序执行；
5、用户涉及的账本数据库和区块变化等写入的动作，都会增加nonce。
6、查询余额等只读操作不会影响nonce；
7、每次发送交易时，nonce必须比上一次动作的nonce大1；
8、交易处理流程：
1、创建交易时，获取当前nonce并加1
2、验证交易时，检查nonce是否正确
3、交易确认后，更新账户nonce



