# Network 网络模块

## 技术栈

*   libp2p
*   serde_json
*   sqlx


## 模块要点
- Network 模块将验证后的交易/铸造等操作，广播到其他节点。
- Network 模块处理钱包与节点之间的交互，如查询余额、查询交易历史、发送交易等。
- Network 模块负责节点之间的网络通信，包括节点发现、连接管理、消息传递等。

## 协议处理与事件处理的关系：
    - 事件处理器接收底层网络事件
    - 当收到消息事件时，调用协议处理器处理具体业务逻辑
    - 协议处理器处理完成后，通过事件通知相关模块

## 与其他模块的交互：
    - 协议处理器需要与 Ledger 模块交互，处理账本相关请求
    - 事件处理器需要与连接管理器交互，处理连接生命周期
    - 两者都需要进行日志记录和错误处理

## 1. 概述

Network 模块负责节点之间的网络通信，包括节点发现、连接管理、消息传递等。它使用 libp2p 库实现点对点网络通信，并提供以下功能：

1. 节点发现 (Discovery):
    - 发现网络中的其他节点，建立连接。
    - 方法：
        - mDNS (Multicast DNS): 适用于局域网内的节点发现。
        - kad + libp2p_identity: 适用于更广泛网络的节点发现。
        - 提供引导节点列表：使用Bootstrap通过预配置的节点列表进行发现
        - 实现节点地址更新机制

2. 连接管理 (Connection Management):
    - Swarm 管理：
        - 基础配置：
            - 使用 SwarmBuilder 构建网络管理器
            - 配置本地节点 ID 和
            - 配置网络行为(NetworkBehaviour)
                - 心跳机制 (Heartbeat)，检测节点在线状态。
            - 配置传输层：
                - TCP + Noise 加密传输
                - Yamux 多路复用
                - 设置流控制参数
            - 配置网络参数：
                - 设置连接超时时间
                - 配置监听地址
                - 设置最大连接数
            
        - 连接管理：
            - 实现连接建立(dial)和断开(disconnect_peer_id)
            - 处理连接事件(ConnectionEstablished/ConnectionClosed)
            - 优雅关闭连接(close_connection)
            
        - 监听管理：
            - 配置监听地址(listen_on)
            - 管理监听器生命周期(remove_listener)
            - 处理新连接事件
            
        - 事件处理：
            - 实现 Stream trait 进行状态轮询
            - 处理 SwarmEvent：
                - 连接事件 (ConnectionEstablished/ConnectionClosed)
                - 监听事件 (NewListenAddr)
                - 行为事件 (Behaviour)
                - 错误事件处理和日志记录
            - 维护网络信息(network_info)
        
4. 协议处理 (Protocol Handling):
    - NetworkBehaviour 实现：
        - 定义 FAICBehaviour 实现网络行为
        - 处理连接事件和协议消息
        - 管理协议状态

    - 业务协议定义：
        - 请求类型（RequestType）：
            - `GetBalance`: 查询余额
            - `SendTransaction`: 发送交易
            - `GetMerkleProof`: 获取默克尔证明
            - `GetBlock`: 获取区块信息
            - `GetBlockByHeight`: 按高度获取区块
            - `GetLatestBlock`: 获取最新区块
            - `GetTransactionStatus`: 查询交易状态
            - `GetTransactionHistory`: 查询交易历史
            - `SyncBlockchain`: 同步区块链
            - `GetNodeInfo`: 获取节点信息
        - 响应类型：对应每种请求的响应结构

    - 消息处理：
        - 使用 serde_json 处理消息序列化
        - 实现请求/响应处理
        - 与 Ledger/Wallet 模块交互


## 2. 模块划分

src/network/
    ├── behaviour.rs      # NetworkBehaviour 实现，包含节点发现和协议处理
    ├── protocol.rs       # 业务协议定义和处理
    ├── config.rs         # 网络配置
    ├── error.rs          # 错误类型定义
    └── mod.rs           # 模块入口，整合各子模块
src/types/
    ├── network.rs  # 网络相关类型定义（请求、响应等）
    └── mod.rs           # 模块入口，整合各子模块

说明：
- behaviour.rs: 整合节点发现和网络行为
- protocol.rs: 处理业务协议和与其他模块的交互
- src/types/network.rs: 定义所有网络相关的数据类型
- config.rs: 处理网络配置
- error.rs: 统一的错误处理
