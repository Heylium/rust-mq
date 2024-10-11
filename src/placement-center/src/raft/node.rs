use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct BrokerNode {
    pub cluster_name: String,
    pub cluster_type: String,
    pub node_id: u64,
    pub node_ip: String,
    pub node_inner_addr: String,
    pub extend: String,
    pub create_time: u128,
}

impl BrokerNode {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}