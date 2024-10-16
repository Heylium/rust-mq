use std::sync::Arc;
use mobc::{Connection, Manager};
use tonic::transport::{Channel, Error};
use common_base::errors::RobustMQError;
use protocol::kv::kv_service_client::KvServiceClient;
use crate::poll::ClientPool;

pub mod call;

mod inner;

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

async fn kv_client(
    client_poll: Arc<ClientPool>,
    addr: String,
) -> Result<Connection<KvServiceManager>, RobustMQError> {
    match client_poll. {  }
}