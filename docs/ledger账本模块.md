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
    *   区块创建`block_create`: 根据DPoS共识机制，创建新的区块。
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

*   **钱包 `Wallet`**:
    *   `address`: 字符串类型，表示钱包的地址。
    *   `available_balance`:  使用Amount类型，表示钱包的可用余额。
    *   `locked_balance`: 使用Amount类型，表示钱包的锁定余额。
    *   `nonce`: i64，表示钱包的动作序号，用于防止重放攻击。
  
*   **钱包操作 `WalletAction`**:
    *   `create_wallet`: 创建钱包。
    *   `delete_wallet`: 删除钱包。
    *   `get_balance`: 查询钱包余额。
    *   `get_transaction_history`: 查询钱包交易历史。

*   **签名类型 `SignatureType`**:
    *   `initiator_signature`: 发起者签名，string类型, secp256k1签名,方法实现文件：src/crypto/signature.rs。

*   **交易类型 `TransactionType`**:
    *   `transfer`: 表示转账交易。
    *   `smart_contract`: 表示智能合约相关的交易。
    *   `dapp`: 表示dapp相关的交易。
    *   `early_donate`: 表示早期捐赠相关的交易，该类型款项用于FAIC早期开发，由开发者手动接收捐赠与分发代币。
    *   未来可以扩展其他类型的交易。

*   **交易信息结构体 `TransactionDetail`**:
    *   `transaction_type`:  交易类型 (`transaction_type`)，表示交易的具体类型。
    *   `from`: 字符串类型，表示交易的发送方地址。
    *   `to`: 字符串类型，表示交易的接收方地址。
    *   `transfer_amount`: 使用Amount类型，表示交易的转账金额。
    *   `locked`：true or false,默认是false，表示该笔交易是否被锁定。
    *   `unlocked_time`: DateTime<Utc>，表示交易解锁的时间戳，默认是0，表示交易未被锁定。当系统时间戳达到或超过 `unlocked_time` 时，该笔交易锁定的金额可以被解锁，变为可用余额。解锁由事件驱动，如+0时区，每天0点检查一次。
    *   `nonce`: i64，表示交易的序号，用于防止重放攻击。
    *   `initiator_signature`: 发起者签名，string类型, secp256k1签名,方法实现文件：src/crypto/signature.rs。
    *   `timestamp`: 时间戳，表示交易发生的时间。
    *   `fee`: 使用Amount类型，表示交易的手续费。
    *   `transaction_hash`: 字节数组，表示交易的哈希值，用于唯一标识一笔交易。该字段在序列化和反序列化时使用十六进制编码。
    *   `transaction_status`: 交易状态，用于标识交易的状态，默认是`await_verify`，如：等待验证、已验证、已确认、已取消、已失败等。
  
*   **交易状态 `transaction_status`**:
    *   `await_verify`: 等待验证，表示交易等待验证。
    *   `confirmed`: 已确认，表示交易已确认。
    *   `pending`: 挂起等待人工处理。当出现unknown状态时，会自动挂起等待人工处理。
    *   `processing`: 处理中，表示交易正在处理中。
    *   `rejected`: 已拒绝，表示交易已拒绝。validator验证不通过的交易，会自动变为`rejected`状态。
    *   `expired`: 已过期，表示交易已过期。默认过期时间是24小时，过期后，交易状态会从`await_verify`变为`expired`。
    *   `unknown`: 未知，表示交易状态未知。

*   **区块 `block`**:
    *   `block_header`: 区块头 (`block_header`)，包含区块的元数据信息。
    *   `block_body`: 区块体(`block_body`)，包含该区块打包的所有数据`data`，如：该区块的所有交易`transaction_payloads_for_block`、智能合约执行结果`smart_contract_payloads_for_block`。

*   **区块头 `block_header`**:
    *   `parent_hash`: 字节数组，表示父区块的哈希值。
    *   `height`: 64 位无符号整数，表示区块的高度。
    *   `timestamp`: 时间戳，表示区块生成的时间。
    *   `merkle_root_hash`: 字节数组，表示区块中所有交易的 Merkle 树根哈希值。该字段在序列化和反序列化时使用十六进制编码。
    *   `validator`: 实现了 `validator` 特征的类型，用于标识区块的验证者**用于标识验证区块的验证者身份**。
    *   `block_header_signature`: 字节数组，表示**区块头签名，由验证者对区块头进行签名生成，用于提供区块合法性和完整性的密码学证明**。
    *   `block_hash`: 字节数组，表示区块的哈希值，用于唯一标识一个区块。该字段在序列化和反序列化时使用十六进制编码。
*   **说明：**  `validator` 字段和 `block_header_signature` 字段共同协作，确保区块的安全性。 `validator` 标识区块的验证者，`block_header_signature` 提供验证者对区块的密码学签名。
  
*   **区块体 `block_body`**:
    *   `transaction_payloads_for_block`: 字节数组，表示区块中所有交易的数据。
    *   `smart_contract_payloads_for_block`: 字节数组，表示区块中所有智能合约的数据。
    *   `version`: 字节，表示区块的版本信息。

*   **区块验证`block_verify`**:
    *   需要创建新的区块时，交易管理模块会从交易池中选择交易，并将这些交易用于构建 Merkle 树，最终将 `merkle_root_hash` 哈希写入区块头`block_header`。
    *   通常区块链的区块验证会包括 验证区块头中的 `merkle_root_hash` 是否与区块体中实际的交易列表构建的 `merkle_root_hash` 一致。

*   **交易哈希计算**:
    *   交易的哈希值是通过对交易的各个字段进行序列化并拼接起来，然后使用 `sha256_concat` 函数进行 SHA256 哈希计算得到的。
    *   具体来说，将交易类型、发送方地址、接收方地址、转账金额、动作序号、交易发起者签名数据（字节数组）、时间戳和手续费依次序列化为字节数组后拼接起来，然后使用 `sha256_concat` 函数进行 SHA256 哈希计算，得到的结果即为交易的哈希值。

*   **区块哈希计算**:
    *   区块的哈希值是通过对区块头的各个字段进行序列化并拼接起来，然后使用 `sha256_concat` 函数进行 SHA256 哈希计算得到的。
    *   具体来说，将父区块哈希、区块高度、时间戳、Merkle 根、验证者身份信息（validator trait）和签名数据 (字节数组)依次序列化为字节数组后拼接起来，然后使用 `sha256_concat` 函数进行 SHA256 哈希计算，得到的结果即为区块的哈希值。

*   **交易池 `transaction_pool`**:
    *   `await_verify_transaction_list`: 等待验证的交易列表，用于存储待验证的交易 (`transaction_detail`)。 验证模块`validator`主动从 await_verify_transaction_list 中获取交易进行验证，验证完成后，再将验证通过的交易放入 verified_transaction_list。verified_transaction_list 通常是预先存在的，用于存储已经验证通过的交易。
    *   `verified_transaction_list`: 已验证的交易列表，用于存储已验证的交易 (`transaction_detail`)。
    *   `add_transaction()`:  添加一笔交易到交易池中。
    *   `remove_transaction()`: 根据交易哈希从交易池中移除一笔交易。
    *   `get_transaction_for_block()`:  从交易池中选择一定数量的交易，用于打包到新的区块中。选择交易的策略可以根据交易费用或其他因素来决定。

*   **`merkle_tree`默克尔树相关说明**:
    *   **数据类型和 `Element` trait**:  在代码实现中，**只有 `block` 结构体实现了 `Element` trait**，用于支持区块数据的序列化和反序列化。  **构建 Merkle 树时，我们直接使用了交易哈希 (`transaction_hash`) 的字节数组 `[u8; 32]` 作为叶子节点，**  并没有直接使用 `transaction_detail`， `block_header` 或 `block_body` 并要求它们实现 `Element` trait。  `Element` trait 的主要目的是为了当 Merkle 树的叶子节点需要存储更复杂的数据结构时，提供序列化和反序列化的支持。
    *   `Element` trait 要求实现以下方法：
        - `byte_len()` - 返回序列化后的字节长度
        - `from_slice(bytes: &[u8]) -> Self` - 从字节切片反序列化
        - `copy_to_slice(&self, bytes: &mut [u8])` - 序列化到字节切片
        - **注意：**  文档原先列出的  `hash(&self) -> Vec<u8>` 方法 **不是 `Element` trait 的要求**。 `Element` trait 主要关注数据的序列化和反序列化。 实际的哈希计算是由 `merkletree` 库自身以及你项目中实现的 `Hash` 结构体 (实现了 `merkletree::hash::Algorithm` trait) 负责的。
    *   我们使用 `serde_json` 进行序列化和反序列化，对于 `block` 结构体，使用 `serde_json` 进行序列化和反序列化。  对于交易哈希，则直接使用其字节数组 `[u8; 32]` 作为 Merkle 树的叶子节点。
    *   **哈希算法**:  我们在构建 Merkle 树时，**使用 SHA-256 哈希算法**。  具体实现上，我们自定义了 `Hash` 结构体 (定义在 `src/crypto/hash.rs` 文件中)，并让它同时实现了 `merkletree` 库要求的 `Algorithm` trait 和 Rust 标准库的 `Hasher` trait。  **实际的 SHA-256 哈希运算是在 `Hash` 结构体实现的 `Hasher::write()` 方法中进行的。**  `merkletree` 库在构建 Merkle 树的过程中，会通过 `Algorithm` trait 来调用我们 `Hash` 结构体的哈希方法，从而间接地使用 SHA-256 算法进行哈希计算。


        *   **Merkle 证明生成 (`generate_proof`):**  当需要向轻客户端 (例如：手机钱包应用) 证明某个交易确实被包含在某个区块中时，我们需要生成 Merkle 证明。  `generate_proof` 功能的目标是：
            1.  **输入:**  交易哈希 (`transaction_hash`)。

            2.  **处理:**  在已构建的 merkletree中，找到该交易哈希对应的叶子节点，并利用 `merkletree` 库的 `generate_proof` 函数，生成从该叶子节点到 Merkle 根的 Merkle 证明路径。  这个证明路径通常包含一组哈希值，用于在验证时重建 Merkle 根。
            3.  **输出:**  `MerkleProof` 对象，其中包含了验证该交易所需的 Merkle 证明数据。


        *   **Merkle 证明验证 (`verify_proof`):**  轻客户端收到 Merkle 证明后，需要验证该证明的有效性，以确认交易确实存在于对应的区块中。 `verify_proof` 功能的目标是：
            1.  **输入:**  `MerkleProof` 对象 (由 `generate_proof` 生成)、交易哈希 (`transaction_hash`)、Merkle 根哈希 (`merkle_root_hash`)。
            2.  **处理:**  使用 `merkletree` 库的 `proof::validate` 函数，结合 `MerkleProof` 中的证明路径、交易哈希和 Merkle 根哈希，重新计算 Merkle 根。

            3.  **输出:**  布尔值 (`bool`)，表示验证结果。 `true` 表示验证通过，证明该交易确实包含在 Merkle 根哈希对应的区块中； `false` 表示验证失败，证明可能无效或交易不存在。
        **开发注意事项:**
        *   **依赖库函数:**  务必使用 `merkletree` 库提供的 `generate_proof` 和 `proof::validate` 函数，而不是自行实现证明生成和验证逻辑。  这将确保代码的正确性和效率。
        *   **错误处理:**  在 `generate_proof` 和 `verify_proof` 的实现中，要考虑各种错误情况，例如：交易哈希不存在于 Merkle 树中、证明数据损坏、验证失败等，并进行适当的错误处理和日志记录。
        *   **性能优化:**  对于频繁的证明生成和验证操作，可以考虑优化 Merkle 树的存储和访问方式，例如使用更高效的数据结构或缓存机制。


    *   **Merkle 树持久化 :** 为了在频繁进行 Merkle 树操作或处理大型区块的场景中提高性能，**将 `MerkleTree` 结构持久化**到数据库。
        **功能描述:**
        *   **持久化目的:**  默认情况下，`MerkleTreeManager` 中的 `MerkleTree` 对象是存储在内存中的。  每次程序重启或需要操作 Merkle 树时，都需要重新构建。  对于区块交易数量非常庞大，或者需要频繁进行 Merkle 树操作的场景 (例如：需要为大量交易生成 Merkle 证明)，重新构建 Merkle 树会消耗较多的时间和计算资源。  Merkle 树持久化的目标是将构建好的 `MerkleTree` 结构保存到外部存储介质 (如磁盘文件或数据库) 中，以便在下次需要使用时，可以直接从外部存储加载，避免重复构建，从而提高性能。
        *   **持久化方案 :**
            1.  **数据库持久化:**  ，可以将 `MerkleTree` 的节点数据存储到`sqlx`数据库表中。  需要设计合适的数据库表结构来表示 Merkle 树的结构和节点信息。

        **开发注意事项:**
        *   **序列化/反序列化:**  选择合适的序列化库和格式，需要考虑序列化和反序列化的性能、数据大小、跨语言兼容性等因素。
        *   **数据一致性:**  在持久化和加载 Merkle 树的过程中，需要确保数据的一致性和完整性。  例如，在多线程或多进程环境下，需要考虑并发访问和修改持久化数据的同步问题。
        *   **存储空间:**  持久化 Merkle 树会占用额外的存储空间。  需要评估存储空间的需求，并根据实际情况选择合适的持久化方案。



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



