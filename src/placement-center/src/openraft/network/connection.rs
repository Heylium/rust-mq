use std::future::Future;
use std::sync::Arc;
use bincode::{deserialize, serialize};
use mobc::Connection;
use openraft::error::{InstallSnapshotError, RPCError, RaftError};
use openraft::network::RPCOption;
use openraft::raft::{AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest, InstallSnapshotResponse, VoteRequest, VoteResponse};
use openraft::RaftNetwork;
use clients::placement::openraft::OpenRaftServiceManager;
use clients::poll::ClientPool;
use common_base::errors::RobustMQError;
use protocol::openraft::{AppendRequest, SnapshotRequest};
use crate::openraft::error::to_error;
use crate::openraft::raft_node::NodeId;
use crate::openraft::typeconfig::TypeConfig;

pub struct NetworkConnection {
    addr: String,
    client_poll: Arc<ClientPool>,
    target: NodeId,
}

impl NetworkConnection {

    pub fn new(addr: String, client_poll: Arc<ClientPool>, target: NodeId) -> Self {
        NetworkConnection {
            addr,
            client_poll,
            target
        }
    }

    async fn c(&mut self) -> Result<Connection<OpenRaftServiceManager>, RobustMQError> {
        Ok(self
            .client_poll
            .placement_center_openraft_services_client(self.addr.clone())
            .await?
        )
    }
}

impl RaftNetwork<TypeConfig> for NetworkConnection {
    async fn append_entries(
        &mut self,
        req: AppendEntriesRequest<TypeConfig>,
        _option: RPCOption
    ) -> Result<AppendEntriesResponse<TypeConfig>, RPCError<TypeConfig, RaftError<TypeConfig>>>
    {
        let mut c = match self.c().await {
            Ok(conn) => conn,
            Err(e) => return Err(to_error(e)),
        };

        let value = match serialize(&req) {
            Ok(data) => data,
            Err(e) => return Err(to_error(RobustMQError::CommonError(e.to_string()))),
        };

        let request = AppendRequest { value };

        let reply = match c.append(request).await {
            Ok(reply) => reply.into_inner(),
            Err(e) => return Err(to_error(RobustMQError::CommonError(e.to_string()))),
        };

        let result = match deserialize(&reply.value) {
            Ok(data) => data,
            Err(e) => return Err(to_error(RobustMQError::CommonError(e.to_string()))),
        };

        Ok(result)
    }

    async fn install_snapshot(
        &mut self,
        req: InstallSnapshotRequest<TypeConfig>,
        _option: RPCOption
    ) -> Result<
        InstallSnapshotResponse<TypeConfig>,
        RPCError<TypeConfig, RaftError<TypeConfig, InstallSnapshotError>>,
    >
    {
        tracing::debug!(req = debug(&req), "install_snapshot");

        let mut c = match self.c().await {
            Ok(conn) => conn,
            Err(e) => return Err(to_error(e)),
        };

        let value = match serialize(&req) {
            Ok(data) => data,
            Err(e) => return Err(to_error(RobustMQError::CommonError(e.to_string()))),
        };

        let request = SnapshotRequest { value };

        let reply = match c.snapshot(request).await {
            Ok(reply) => reply.into_inner(),
            Err(e) => return Err(to_error(RobustMQError::CommonError(e.to_string()))),
        };
        let result = match deserialize(&reply.value) {
            Ok(data) => data,
            Err(e) => return Err(to_error(RobustMQError::CommonError(e.to_string()))),
        };

        Ok(result)
    }

    async fn vote(
        &mut self,
        req: VoteRequest<TypeConfig>,
        _option: RPCOption
    ) -> Result<VoteResponse<TypeConfig>, RPCError<TypeConfig, RaftError<TypeConfig>>>
    {
        tracing::debug!(req = debug(&req), "vote");
        let mut c = match self.c().await {
            Ok(conn) => conn,
            Err(e) => return Err(to_error(e)),
        };

        let value = match serialize(&req) {
            Ok(data) => data,
            Err(e) => return Err(to_error(RobustMQError::CommonError(e.to_string()))),
        };

        let request = protocol::openraft::VoteRequest { value };

        let reply = match c.vote(request).await {
            Ok(reply) => reply.into_inner(),
            Err(e) => return Err(to_error(RobustMQError::CommonError(e.to_string()))),
        };
        let result = match deserialize(&reply.value) {
            Ok(data) => data,
            Err(e) => return Err(to_error(RobustMQError::CommonError(e.to_string()))),
        };

        Ok(result)
    }
}