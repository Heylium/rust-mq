use std::collections::HashMap;
use log::__private_api::loc;
use log::info;
use raft::StateRole;
use common_base::config::placement_center::placement_center_conf;
use protocol::common::ClusterType;
use crate::raft::node::BrokerNode;

#[derive(PartialEq, Default, Debug, Eq, PartialOrd, Ord, Clone)]
pub enum NodeState {
    #[default]
    Running,
    Starting,
    #[allow(dead_code)]
    Stopping,
    #[allow(dead_code)]
    Stop,
}


pub struct RaftGroupMetadata {
    pub local: BrokerNode,
    pub leader: Option<BrokerNode>,
    pub state: NodeState,
    pub raft_role: StateRole,
    pub peers: HashMap<u64, BrokerNode>,
}

impl RaftGroupMetadata {
    pub fn new() -> RaftGroupMetadata {
        let config = placement_center_conf();
        let mut local = BrokerNode::default();
        local.cluster_type = ClusterType::PlacementCenter.as_str_name().to_string();
        local.cluster_name = config.cluster_name.clone();
        local.node_inner_addr = format!("{}:{}", config.addr.clone(), config.grpc_port);
        local.node_ip = config.addr.clone();
        local.node_id = config.node_id;

        let mut peers = HashMap::new();
        for (node_id, addr) in config.nodes.clone() {
            let (ip, _) = addr.as_str().unwrap().split_once(":").unwrap();
            let id: u64 = node_id.to_string().trim().parse().unwrap();
            let mut node = BrokerNode::default();

            node.cluster_type = ClusterType::PlacementCenter.as_str_name().to_string();
            node.cluster_name = config.cluster_name.clone();
            node.node_inner_addr = addr.to_string();
            node.node_ip = ip.to_string();
            node.node_id = id;
            peers.insert(id, node);
        }

        RaftGroupMetadata {
            local,
            leader: None,
            raft_role: StateRole::Follower,
            state: NodeState::Starting,
            peers,
        }
    }

    pub fn get_node_by_id(&self, id: u64) -> Option<&BrokerNode> {
        self.peers.get(&id)
    }

    pub fn add_peer(&mut self, id: u64, node: BrokerNode) {
        info!("add peer node:{:?}", node);
        self.peers.insert(id, node);
    }

    pub fn remove_peer(&mut self, id: u64) {
        info!("remove peer node id:{:?}", id);
        self.peers.remove(&id);
    }

    pub fn is_leader(&self) -> bool {
        self.raft_role == StateRole::Leader
    }

    pub fn set_role(&mut self, role: StateRole) {
        self.raft_role = role;
    }

    pub fn node_ids(&self) -> Vec<u64> {
        let mut voters = Vec::new();
        for (id, _) in self.peers.iter() {
            voters.push(*id);
        }
        voters
    }

    pub fn leader_addr(&self) -> String {
        if let Some(leader) = self.leader.clone() {
            return leader.node_inner_addr;
        }
        "".to_string()
    }

    pub fn leader_alive(&self) -> bool {
        if let Some(_) = self.leader.clone() {
            return true;
        }
        false
    }
}