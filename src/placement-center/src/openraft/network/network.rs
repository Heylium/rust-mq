use std::future::Future;
use std::sync::Arc;
use openraft::RaftNetworkFactory;
use clients::poll::ClientPool;
use crate::openraft::network::connection::NetworkConnection;
use crate::openraft::raft_node::{Node, NodeId};
use crate::openraft::typeconfig::TypeConfig;

pub struct Network {
    client_poll: Arc<ClientPool>,
}

impl Network {
    pub fn new(client_poll: Arc<ClientPool>) -> Network {
        Network {
            client_poll,
        }
    }
}

// NOTE: This could be implemented also on `Arc<ExampleNetwork>`, but since it's empty, implemented
// directly.
impl RaftNetworkFactory<TypeConfig> for Network {
    type Network = NetworkConnection;

    #[tracing::instrument(level = "debug", skip_all)]
    async fn new_client(&mut self, target: NodeId, node: &Node) -> Self::Network {
        let addr = format!("{}", node.rpc_addr);
        NetworkConnection::new(addr, self.client_poll.clone(), target)
    }
}