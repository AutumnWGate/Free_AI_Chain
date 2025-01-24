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
                    Some(id) => Ok(NetworkConfig::new(id)),
                    None => NetworkConfig::default(),
                }?;
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

