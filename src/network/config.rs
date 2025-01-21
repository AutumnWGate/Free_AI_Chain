use libp2p::{Multiaddr, PeerId}; //用于P2P网络通信
use serde::{Deserialize, Serialize}; //用于序列化和反序列化
use std::fs;
use std::time::Duration; //用于时间操作
use toml; //用于TOML格式解析

// 自定义序列化和反序列化模块
pub mod serde_peer_id {
    use libp2p::PeerId;
    use serde::Deserialize;

    // 序列化 PeerId 为字符串
    pub fn serialize<S>(peer_id: &PeerId, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&peer_id.to_string())
    }

    // 反序列化字符串为 PeerId
    pub fn deserialize<'de, D>(deserializer: D) -> Result<PeerId, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

mod serde_multiaddr {
    use libp2p::Multiaddr;
    use serde::{Deserialize, Serialize, Serializer};

    // 序列化 Multiaddr 为字符串
    pub fn serialize<S>(multiaddrs: &Vec<Multiaddr>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let strings: Vec<String> = multiaddrs.iter().map(|ma| ma.to_string()).collect();
        strings.serialize(serializer)
    }

    // 反序列化字符串为 Multiaddr
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Multiaddr>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let strings: Vec<String> = Vec::deserialize(deserializer)?;
        strings
            .into_iter()
            .map(|s| s.parse().map_err(serde::de::Error::custom))
            .collect()
    }
}

mod serde_bootstrap_nodes {
    use libp2p::{Multiaddr, PeerId};
    use serde::ser::SerializeTuple;
    use serde::Deserialize;

    // 序列化引导节点列表 (PeerId, Multiaddr) 为元组数组
    pub fn serialize<S>(nodes: &[(PeerId, Multiaddr)], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_tuple(nodes.len())?;
        for (peer_id, multiaddr) in nodes {
            // 将 PeerId 和 Multiaddr 转换为字符串后，作为元组的元素进行序列化
            seq.serialize_element(&(peer_id.to_string(), multiaddr.to_string()))?;
        }
        seq.end()
    }

    // 从元组数组反序列化引导节点列表
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<(PeerId, Multiaddr)>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // 反序列化为字符串元组的向量
        let vec = Vec::<(String, String)>::deserialize(deserializer)?;
        // 将每个字符串元组转换为 (PeerId, Multiaddr)
        vec.into_iter()
            .map(|(pid_str, ma_str)| {
                let pid = pid_str.parse().map_err(serde::de::Error::custom)?;
                let ma = ma_str.parse().map_err(serde::de::Error::custom)?;
                Ok((pid, ma))
            })
            .collect()
    }
}

mod serde_duration_secs {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    // 序列化 Duration 为 u64 表示的秒数
    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    // 从 u64 表示的秒数反序列化 Duration
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

/// 网络配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    // 本地节点的 PeerId，使用自定义的序列化和反序列化方法
    #[serde(with = "serde_peer_id")]
    pub local_peer_id: PeerId,
    // 监听地址列表，使用自定义的序列化和反序列化方法
    #[serde(with = "serde_multiaddr")]
    pub listen_addresses: Vec<Multiaddr>,
    // 引导节点列表，使用自定义的序列化和反序列化方法
    #[serde(with = "serde_bootstrap_nodes")]
    pub bootstrap_nodes: Vec<(PeerId, Multiaddr)>,
    // 最大连接数
    pub max_connections: u32,
    // 连接超时时间，使用自定义的序列化和反序列化方法
    #[serde(with = "serde_duration_secs")]
    pub connection_timeout: Duration,
    // 心跳间隔时间，使用自定义的序列化和反序列化方法
    #[serde(with = "serde_duration_secs")]
    pub heartbeat_interval: Duration,
}

impl NetworkConfig {
    /// 创建一个新的 NetworkConfig，允许指定 PeerId
    pub fn new(local_peer_id: PeerId) -> Self {
        NetworkConfig {
            local_peer_id,
            listen_addresses: vec!["/ip4/0.0.0.0/tcp/0".parse().unwrap()],
            bootstrap_nodes: vec![],
            max_connections: 100,
            connection_timeout: Duration::from_secs(10),
            heartbeat_interval: Duration::from_secs(60),
        }
    }
}

// 默认实现 NetworkConfig 的默认值
impl Default for NetworkConfig {
    fn default() -> Self {
        let config = NetworkConfig {
            local_peer_id: PeerId::random(), // 随机生成 PeerId
            listen_addresses: vec!["/ip4/0.0.0.0/tcp/0".parse().unwrap()], // 监听所有IPv4地址的0号端口
            bootstrap_nodes: vec![],                                       // 初始为空
            max_connections: 100,                                          // 最大连接数
            connection_timeout: Duration::from_secs(10),                   // 连接超时时间
            heartbeat_interval: Duration::from_secs(60),                   // 心跳间隔时间
        };
        println!("网络配置默认值: {:?}", config);
        config
    }
}

// 自定义错误类型
#[derive(Debug)]
pub enum NetworkConfigError {
    IoError(std::io::Error),              // 文件操作错误
    TomlError(toml::de::Error),           // TOML解析错误
    TomlSerializeError(toml::ser::Error), // TOML序列化错误
}

// 为 NetworkConfigError 实现 Display trait，用于打印错误信息
impl std::fmt::Display for NetworkConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            NetworkConfigError::IoError(e) => write!(f, "IO error: {}", e),
            NetworkConfigError::TomlError(e) => write!(f, "TOML deserialization error: {}", e),
            NetworkConfigError::TomlSerializeError(e) => {
                write!(f, "TOML serialization error: {}", e)
            }
        }
    }
}

// 为 NetworkConfigError 实现 Error trait
impl std::error::Error for NetworkConfigError {}

// 实现从 std::io::Error 到 NetworkConfigError 的转换
impl From<std::io::Error> for NetworkConfigError {
    fn from(err: std::io::Error) -> Self {
        NetworkConfigError::IoError(err)
    }
}

// 实现从 toml::de::Error 到 NetworkConfigError 的转换
impl From<toml::de::Error> for NetworkConfigError {
    fn from(err: toml::de::Error) -> Self {
        NetworkConfigError::TomlError(err)
    }
}

// 实现从 toml::ser::Error 到 NetworkConfigError 的转换
impl From<toml::ser::Error> for NetworkConfigError {
    fn from(err: toml::ser::Error) -> Self {
        NetworkConfigError::TomlSerializeError(err)
    }
}

impl NetworkConfig {
    /// 从文件加载网络配置
    ///
    /// # Arguments
    ///
    /// * `path` - 配置文件的路径
    ///
    /// # Returns
    ///
    /// `Result<NetworkConfig, NetworkConfigError>` - 返回加载的配置或错误    

    pub fn load_from_file(path: &str) -> Result<Self, NetworkConfigError> {
        // 读取配置文件内容到字符串
        let config_str = fs::read_to_string(path)?;
        // 将字符串反序列化为 NetworkConfig 结构体
        let config: NetworkConfig = toml::from_str(&config_str)?;
        // 返回加载的配置
        Ok(config)
    }

    /// 保存网络配置到文件
    ///
    /// # Arguments
    ///
    /// * `path` - 要保存到的文件路径
    ///
    /// # Returns
    ///
    /// `Result<(), NetworkConfigError>` - 返回成功或错误
    pub fn save_to_file(&self, path: &str) -> Result<(), NetworkConfigError> {
        // 将 NetworkConfig 结构体序列化为字符串
        let config_str = toml::to_string(self)?;
        // 将字符串写入文件
        std::fs::write(path, config_str)?;
        // 返回成功
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_load_and_save_config() {
        // 创建一个临时文件
        let temp_file = "temp_config.toml";

        // 创建一个默认配置
        let config = NetworkConfig::default();

        // 保存配置到临时文件
        config.save_to_file(temp_file).unwrap();

        // 从临时文件加载配置
        let loaded_config = NetworkConfig::load_from_file(temp_file).unwrap();

        // 验证加载的配置与原始配置相同
        assert_eq!(config.local_peer_id, loaded_config.local_peer_id);
        assert_eq!(config.listen_addresses, loaded_config.listen_addresses);
        assert_eq!(config.bootstrap_nodes, loaded_config.bootstrap_nodes);
        assert_eq!(config.max_connections, loaded_config.max_connections);
        assert_eq!(config.connection_timeout, loaded_config.connection_timeout);
        assert_eq!(config.heartbeat_interval, loaded_config.heartbeat_interval);

        // 删除临时文件
        fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_load_invalid_config() {
        // 创建一个无效的配置文件
        let invalid_config = "invalid_config.toml";
        fs::write(invalid_config, "invalid toml content").unwrap();

        // 尝试加载无效的配置
        let result = NetworkConfig::load_from_file(invalid_config);

        // 验证加载结果为错误
        assert!(result.is_err());

        // 删除无效的配置文件
        fs::remove_file(invalid_config).unwrap();
    }

    #[test]
    fn test_default_config() {
        let config = NetworkConfig::default();

        // 检查 listen_addresses
        assert_eq!(
            config.listen_addresses,
            vec!["/ip4/0.0.0.0/tcp/0".parse().unwrap()]
        );

        // 检查其他字段
        assert_eq!(config.bootstrap_nodes, vec![]);
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.connection_timeout, Duration::from_secs(10));
        assert_eq!(config.heartbeat_interval, Duration::from_secs(60));
    }
}
