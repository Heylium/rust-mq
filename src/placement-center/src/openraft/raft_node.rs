use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::sync::{Arc};
use log::info;
use openraft::{Config, Raft};
use openraft::error::{Fatal, InitializeError, RaftError};
use tokio::sync::RwLock;
use clients::poll::ClientPool;
use common_base::config::placement_center::placement_center_conf;
use crate::openraft::network::network::Network;
use crate::openraft::sotre::new_storage;
use crate::openraft::typeconfig::TypeConfig;

pub type NodeId = u64;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
pub struct Node {
    pub node_id: NodeId,
    pub rpc_addr: String,
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Node {{ rpc_addr: {}, node_id: {}, }}",
            self.rpc_addr,
            self.node_id,
        )
    }
}

pub mod typ {
    use std::convert::Infallible;
    use crate::openraft::typeconfig::TypeConfig;

    pub type Entry = openraft::Entry<TypeConfig>;

    pub type RaftError<E = Infallible> = openraft::error::RaftError<TypeConfig, E>;

    pub type RPCError<E = Infallible> = openraft::error::RPCError<TypeConfig, RaftError<E>>;

    pub type ClientWriteError = openraft::error::ClientWriteError<TypeConfig>;

    pub type CheckIsLeaderError = openraft::error::CheckIsLeaderError<TypeConfig>;

    pub type ForwardToLeader = openraft::error::ForwardToLeader<TypeConfig>;

    pub type InitializeError = openraft::error::InitializeError<TypeConfig>;

    pub type ClientWriteResponse = openraft::raft::ClientWriteResponse<TypeConfig>;
}

pub type ExampleRaft = openraft::Raft<TypeConfig>;

pub async fn start_openraft_node(raft_node: ExampleRaft) {
    let conf = placement_center_conf();
    let mut nodes = BTreeMap::new();
    for (node_id, addr) in conf.nodes.clone() {
        let mut addr = addr.to_string();
        addr = addr.replace("\"", "");
        let node = Node {
            rpc_addr: addr,
            node_id: node_id.parse().unwrap(),
        };
        nodes.insert(node.node_id, node);
    }

    info!("Raft Nodes:{:?}", nodes);
    let init_node_id = calc_init_node(&nodes);
    if init_node_id == conf.node_id {
        match raft_node.is_initialized().await {
            Ok(flag) => {
                info!("Whether nodes should be initialized, flag={}", flag);
                if !flag {
                    match raft_node.initialize(nodes.clone()).await {
                        Ok(_) => {
                            info!("Node {:?} was initialized successfully", nodes);
                        }
                        Err(e) => {
                            panic!("openraft init fail,{}", e.to_string());
                        }
                    }
                }
            }
            Err(e) => {
                panic!("openraft initialized fail,{}", e.to_string());
            }
        }
    }
}

pub fn calc_init_node(nodes: &BTreeMap<u64, Node>) -> u64 {
    let mut node_ids: Vec<u64> = nodes.keys().map(|x| x.clone()).collect();
    node_ids.sort();
    node_ids.first().unwrap().clone()
}

pub async fn create_raft_node(
    client_poll: Arc<ClientPool>,
) -> (Raft<TypeConfig>, Arc<RwLock<BTreeMap<String, String>>>) {
    let config = Config {
        heartbeat_interval: 250,
        election_timeout_min: 299,
        ..Default::default()
    };

    let config = Arc::new(config.validate().unwrap());
    let conf = placement_center_conf();
    let path = format!("{}/_engine_storage", conf.data_path.clone());
    let dir = Path::new(&path);
    let (log_store, state_machine_store) = new_storage(&dir).await;
    let kvs = state_machine_store.data.kvs.clone();

    let network = Network::new(client_poll);
    let raft = openraft::Raft::new(
        conf.node_id,
        config.clone(),
        network,
        log_store,
        state_machine_store,
    ).await.unwrap();

    (raft, kvs)
}