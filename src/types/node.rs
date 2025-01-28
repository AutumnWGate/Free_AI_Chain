use libp2p::{Multiaddr, PeerId};
use serde::{Deserialize, Serialize};

/// 节点信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeInfo {
    /// 节点的 PeerId
    #[serde(with = "crate::network::config::serde_peer_id")]
    pub peer_id: PeerId,
    /// 节点的地址列表
    pub addresses: Vec<Multiaddr>,
    /// 节点是否在线
    pub is_online: bool,
    /// 节点版本
    pub node_version: String,
}
