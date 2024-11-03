use bincode::{deserialize, serialize};
use openraft::Raft;
use tonic::{Request, Response, Status};
use protocol::openraft::open_raft_service_server::OpenRaftService;
use protocol::openraft::{AppendReply, AppendRequest, SnapshotReply, SnapshotRequest, VoteReply, VoteRequest};
use crate::openraft::typeconfig::TypeConfig;

pub struct GrpcOpenRaftServices {
    raft_node: Raft<TypeConfig>,
}

impl GrpcOpenRaftServices {
    pub fn new(raft_node: Raft<TypeConfig>) -> Self {
        GrpcOpenRaftServices { raft_node }
    }
}

#[tonic::async_trait]
impl OpenRaftService for GrpcOpenRaftServices {
    async fn vote(&self, request: Request<VoteRequest>) -> Result<Response<VoteReply>, Status> {
        let req = request.into_inner();
        let vote_data = deserialize(&req.value).unwrap();
        let res = match self.raft_node.vote(vote_data).await {
            Ok(data) => data,
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        };

        let mut reply = VoteReply::default();
        reply.value = match serialize(&res) {
            Ok(data) => data,
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        };

        Ok(Response::new(reply))
    }

    async fn append(&self, request: Request<AppendRequest>) -> Result<Response<AppendReply>, Status> {
        let req = request.into_inner();
        let vote_data = deserialize(&req.value).unwrap();
        let res = match self.raft_node.append_entries(vote_data).await {
            Ok(data) => data,
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        };
        let mut reply = AppendReply::default();
        reply.value = match serialize(&res) {
            Ok(data) => data,
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        };
        Ok(Response::new(reply))
    }

    async fn snapshot(&self, request: Request<SnapshotRequest>) -> Result<Response<SnapshotReply>, Status> {
        let req = request.into_inner();
        let vote_data = deserialize(&req.value).unwrap();
        let res = match self.raft_node.install_snapshot(vote_data).await {
            Ok(data) => data,
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        };

        let mut reply = SnapshotReply::default();
        reply.value = match serialize(&res) {
            Ok(data) => data,
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        };
        Ok(Response::new(reply))
    }
}