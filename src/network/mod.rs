pub mod config;
pub mod error;
pub mod protocol;

use crate::network::error::Error;
use crate::types::node::NodeInfo;

pub async fn get_node_info() -> Result<NodeInfo, Error> {
    // TODO: 实现获取节点信息逻辑
    unimplemented!("get_node_info not implemented")
}
