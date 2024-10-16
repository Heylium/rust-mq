use mobc::Manager;
use tonic::transport::Channel;
use protocol::openraft::open_raft_service_client::OpenRaftServiceClient;

pub mod call;
mod inner;

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
    type Error = ();

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        todo!()
    }

    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        todo!()
    }
}