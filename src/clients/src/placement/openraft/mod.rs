use std::sync::Arc;
use common_base::errors::RobustMQError;
use mobc::{Connection, Manager};
use protocol::openraft::open_raft_service_client::OpenRaftServiceClient;
use tonic::transport::Channel;
use crate::placement::openraft::inner::{inner_append, inner_snapshot, inner_vote};
use crate::placement::PlacementCenterInterface;
use crate::poll::ClientPool;

pub mod call;
mod inner;

pub(crate) async fn openraft_interface_call(
    interface: PlacementCenterInterface,
    client_pool: Arc<ClientPool>,
    addr: String,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match openraft_client(client_pool.clone(), addr.clone()).await {
        Ok(client) => {
            let result = match interface {
                PlacementCenterInterface::Vote => inner_vote(client, request.clone()).await,
                PlacementCenterInterface::Append => inner_append(client, request.clone()).await,
                PlacementCenterInterface::Snapshot => inner_snapshot(client, request.clone()).await,
                _ => Err(RobustMQError::CommonError(format!(
                    "openraft service does not support service interface [{:?}]",
                    interface,
                )))
            };
            match result {
                Ok(data) => Ok(data),
                Err(e) => Err(e),
            }
        },
        Err(e) => Err(e),
    }
}

async fn openraft_client(
    client_pool: Arc<ClientPool>,
    addr: String,
) -> Result<Connection<OpenRaftServiceManager>, RobustMQError> {
    match client_pool
        .placement_center_openraft_services_client(addr)
        .await
    {
        Ok(client) => Ok(client),
        Err(e) => Err(e),
    }
}

#[derive(Clone)]
pub struct OpenRaftServiceManager {
    pub addr: String,
}

impl OpenRaftServiceManager {
    pub fn new(addr: String) -> Self {
        Self {
            addr
        }
    }
}

#[tonic::async_trait]
impl Manager for OpenRaftServiceManager {
    type Connection = OpenRaftServiceClient<Channel>;
    type Error = RobustMQError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let mut addr = format!("http://{}", self.addr.clone());

        match OpenRaftServiceClient::connect(addr.clone()).await {
            Ok(client) => Ok(client),
            Err(err) => Err(RobustMQError::CommonError(format!(
                "{},{}",
                err.to_string(),
                addr,
            )))
        }
    }

    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        Ok(conn)
    }
}