use faic_core::network::config::{NetworkConfig, NetworkConfigError};
use libp2p::PeerId;

fn main() -> Result<(), NetworkConfigError> {
    let config_path = "config.toml"; // 默认路径
    load_or_create_config(config_path, None);
    env_logger::init();
    Ok(())
}

fn load_or_create_config(
    config_path: &str,
    peer_id: Option<PeerId>,
) -> Result<(), NetworkConfigError> {
    match NetworkConfig::load_from_file(config_path) {
        Ok(config) => {
            println!("Loaded network config: {:?}", config);
            Ok(())
        }
        Err(err) => match err {
            NetworkConfigError::IoError(ref io_err)
                if io_err.kind() == std::io::ErrorKind::NotFound =>
            {
                println!("Config file not found, creating default config.");
                let default_config = match peer_id {
                    Some(id) => NetworkConfig::new(id),
                    None => NetworkConfig::default(),
                };
                default_config.save_to_file(config_path)?;
                Ok(())
            }
            _ => {
                println!("Error loading config: {}", err);
                Err(err)
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::time::Duration;
    #[test]
    fn test_load_valid_config() {
        // 创建一个有效的配置文件
        let config_path = "test_valid_config.toml";
        let mut file = fs::File::create(config_path).unwrap();
        writeln!(
            file,
            r#"
    local_peer_id = "12D3KooWLyEavPji9n9qaGcoe5j4qoJYkDwHLN5MU7o26d54fVMD"
    listen_addresses = ["/ip4/127.0.0.1/tcp/8080"]
    bootstrap_nodes = []
    max_connections = 50
    connection_timeout = 20
    heartbeat_interval = 30
    "#
        )
        .unwrap();

        // 运行主程序逻辑，传递 None 作为 peer_id
        let result = load_or_create_config(config_path, None);

        // 验证结果
        assert!(result.is_ok());

        // 删除临时文件
        fs::remove_file(config_path).unwrap();
    }

    #[test]
    fn test_config_not_found() {
        // 使用测试专用的文件路径
        let config_path = "test_not_found_config.toml";

        // 确保文件不存在
        fs::remove_file(config_path).unwrap_or(());

        // 使用一个已知的合法 PeerId 字符串
        let fixed_peer_id: PeerId = "12D3KooWLyEavPji9n9qaGcoe5j4qoJYkDwHLN5MU7o26d54fVMD"
            .parse()
            .expect("Failed to parse PeerId");

        // 运行主逻辑，传入测试专用的文件路径和固定的 PeerId
        let result = load_or_create_config(config_path, Some(fixed_peer_id));

        // 检查是否返回 Ok
        assert!(result.is_ok());

        // 检查文件是否被创建
        assert!(fs::metadata(config_path).is_ok());

        // 加载保存的配置
        let loaded_config = NetworkConfig::load_from_file(config_path).unwrap();

        // 检查文件内容是否为默认配置
        assert_eq!(loaded_config.local_peer_id, fixed_peer_id);
        assert_eq!(
            loaded_config.listen_addresses,
            vec!["/ip4/0.0.0.0/tcp/0".parse().unwrap()]
        );
        assert_eq!(loaded_config.bootstrap_nodes, vec![]);
        assert_eq!(loaded_config.max_connections, 100);
        assert_eq!(loaded_config.connection_timeout, Duration::from_secs(10));
        assert_eq!(loaded_config.heartbeat_interval, Duration::from_secs(60));

        // 清理文件
        fs::remove_file(config_path).unwrap();
    }

    #[test]
    fn test_load_invalid_config() {
        // 使用测试专用的文件路径
        let config_path = "test_invalid_config.toml";

        // 创建无效的 TOML 文件
        let mut file = fs::File::create(config_path).unwrap();
        writeln!(file, "invalid toml content").unwrap();

        // 运行主逻辑，传入测试专用的文件路径和 None 作为 peer_id
        let result = load_or_create_config(config_path, None);

        // 检查是否返回错误
        assert!(result.is_err());

        // 清理文件
        fs::remove_file(config_path).unwrap();
    }
}
