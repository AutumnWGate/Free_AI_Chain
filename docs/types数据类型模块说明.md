# Type 数据类型模块说明

## 1. 模块结构

src/types/
├── mod.rs # 模块导出
├── amount.rs # 金额类型
├── block.rs # 区块类型
├── ledger.rs # 账本类型
├── merkletree.rs # 默克尔树类型
├── message.rs # 消息类型
├── node.rs # 节点类型
├── transaction.rs # 交易类型
└── wallet.rs # 钱包类型

## 2. 模块说明

### 2.1 amount.rs - 金额类型
- **主要类型**：`Amount`
- **功能**：处理FAIC代币的数量表示和计算
- **关键特性**：
  - 精度：8位小数
  - 最小单位：1 (0.00000001 FAIC)
  - 最大数量：2^128 - 1
- **主要方法**：
  - `from_biguint(value: BigUint)`: 从BigUint创建Amount，检查是否超过最大值
  - `from_str(value: &str)`: 从字符串创建Amount，支持小数点表示
  - `to_string()`: 将Amount转换为带8位小数的字符串表示
  - `to_bytes_be()`: 获取金额的大端序字节表示
  - 实现了加减乘除等基本运算操作

### 2.2 block.rs - 区块类型
- **主要类型**：`Block`, `BlockHeader`
- **功能**：定义区块链中的区块结构
- **关键组件**：
  - 区块头：包含元数据
  - 交易列表：包含区块中的所有交易
- **主要方法**：
  - `calculate_block_hash(header: &BlockHeader)`: 计算区块哈希值
  - `Default::default()`: 创建默认区块，用于创世区块
  - 实现了`AsRef<[u8]>`用于序列化
  - 实现了`Ord`和`PartialOrd`用于区块排序

### 2.3 ledger.rs - 账本类型
- **主要类型**：`LedgerState`, `WalletManagement`, `BlockManagement`, `TransactionManagement`
- **功能**：管理整个账本的状态
- **主要组件**：
  - `WalletManagement`: 管理钱包记录和操作
  - `BlockManagement`: 管理区块链和最新区块
  - `TransactionManagement`: 管理待处理和已确认的交易
- **验证枚举**：
  - `TransactionVerifyResult`: 交易验证结果（有效、余额不足等）
  - `BlockVerifyResult`: 区块验证结果（有效、父哈希无效等）

### 2.4 merkletree.rs - 默克尔树类型
- **功能**：实现区块的默克尔树功能
- **主要实现**：
  - `Element` trait实现：
    - `byte_len()`: 获取序列化后的字节长度
    - `from_slice()`: 从字节切片创建元素
    - `copy_to_slice()`: 将元素复制到字节切片
- **错误处理**：`MerkleTreeError`

### 2.5 message.rs - 消息类型
- **主要类型**：`MessageType`, `Request`, `Response`
- **功能**：定义节点间通信的消息格式
- **请求类型**：
  - 获取节点信息
  - 获取余额
  - 发送交易
  - 获取默克尔证明
  - 获取区块信息
- **响应类型**：
  - 对应每种请求的响应
  - 错误响应

### 2.6 node.rs - 节点类型
- **主要类型**：`NodeInfo`
- **功能**：定义节点信息
- **关键字段**：
  - `peer_id`: 节点唯一标识
  - `addresses`: 节点网络地址列表
  - `is_online`: 节点在线状态
  - `node_version`: 节点版本信息

### 2.7 transaction.rs - 交易类型
- **主要类型**：`Transaction`, `TransactionPool`
- **功能**：定义交易结构和交易池
- **主要方法**：
  - `new()`: 创建新交易，自动计算交易哈希
  - `verify_signature()`: 验证交易签名
  - `calculate_transaction_hash()`: 计算交易哈希值
- **交易池功能**：
  - `add_transaction()`: 添加交易到池中
  - `remove_transaction()`: 从池中移除交易
  - `get_transactions_for_block()`: 获取用于打包区块的交易
  - `sort_by_fee()`: 按照交易费排序
  - `remove_expired_transactions()`: 清理过期交易

### 2.8 wallet.rs - 钱包类型
- **主要类型**：`Wallet`
- **功能**：管理用户钱包
- **主要方法**：
  - `new()`: 创建新钱包，返回钱包实例和助记词
  - `from_mnemonic()`: 从助记词恢复钱包
- **关键字段**：
  - `address`: 钱包地址
  - `balance`: 账户余额
  - `nonce`: 交易序号

## 3. 类型间的关系

1. **交易相关**：
   - `Transaction` 使用 `Amount` 表示转账金额
   - `Transaction` 包含在 `Block` 中
   - `TransactionPool` 管理待处理的交易

2. **区块相关**：
   - `Block` 包含 `Transaction` 列表
   - `Block` 使用 `MerkleTree` 生成默克尔根

3. **账本相关**：
   - `LedgerState` 管理 `Block`、`Transaction` 和 `Wallet`
   - `Wallet` 使用 `Amount` 表示余额

4. **网络通信**：
   - `Message` 用于传输 `Block`、`Transaction` 等数据
   - `NodeInfo` 用于节点间的信息交换

## 4. 错误处理

所有模块都实现了自己的错误类型：
- `AmountError`
- `BlockError`
- `TransactionError`
- `MerkleTreeError`

这些错误类型都实现了 `std::error::Error` trait，并使用 `thiserror` 进行错误处理。