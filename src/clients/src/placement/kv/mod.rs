use std::sync::Arc;
use mobc::{Connection, Manager};
use tonic::transport::{Channel, Error};
use common_base::errors::RobustMQError;
use protocol::kv::kv_service_client::KvServiceClient;
use crate::placement::PlacementCenterInterface;
use crate::poll::ClientPool;

pub mod call;

mod inner;

pub(crate) async fn kv_interface_call(
    interface: PlacementCenterInterface,
    client_poll: Arc<ClientPool>,
    addr: String,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match kv_client(client_poll.clone(), addr.clone()).await {
        Ok(client) => {
            let result = match interface {
                PlacementCenterInterface::Set => inner::inner_set(client, request.clone()).await,
                PlacementCenterInterface::Delete => inner::inner_delete(client, request.clone()).await,
                PlacementCenterInterface::Get => inner::inner_get(client, request.clone()).await,
                PlacementCenterInterface::Exists => inner::inner_exists(client, request.clone()).await,
                _ => Err(RobustMQError::CommonError(format!(
                    "kv service does not support service interfaces {:?}",
                    interface
                ))),
            };
            match result {
                Ok(data) => Ok(data),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e)
    }
}

async fn kv_client(
    client_poll: Arc<ClientPool>,
    addr: String,
) -> Result<Connection<KvServiceManager>, RobustMQError> {
    match client_poll.placement_center_kv_services_client(addr).await {
        Ok(client) => {
            Ok(client)
        }
        Err(e) => {
            Err(e)
        }
    }
}

#[derive(Clone)]
pub struct KvServiceManager {
    pub addr: String,
}

impl KvServiceManager {
    pub fn new(addr: String) -> Self {
        Self {
            addr,
        }
    }
}

#[tonic::async_trait]
impl Manager for KvServiceManager {
    type Connection = KvServiceClient<Channel>;

    type Error = RobustMQError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match KvServiceClient::connect(format!("http://{}", self.addr.clone())).await {
            Ok(client) => Ok(client),
            Err(err) => Err(RobustMQError::CommonError(format!(
                "{},{}",
                err.to_string(),
                self.addr.clone(),
            ))),
        }
    }

    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        Ok(conn)
    }
}
