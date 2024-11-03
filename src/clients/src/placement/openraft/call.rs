use std::future::Future;
use std::sync::Arc;
use prost::{DecodeError, Message};
use common_base::errors::RobustMQError;
use protocol::openraft::{AppendReply, AppendRequest, SnapshotReply, SnapshotRequest, VoteReply, VoteRequest};
use crate::placement::{retry_call, PlacementCenterInterface, PlacementCenterService};
use crate::poll::ClientPool;

pub async fn placement_openraft_vote(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: VoteRequest,
) -> Result<VoteReply, RobustMQError> {
    let resquest_data = VoteRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::OpenRaft,
        PlacementCenterInterface::Vote,
        client_pool,
        addrs,
        resquest_data,
    ).await {
        Ok(data) => match VoteReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(RobustMQError::CommonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn placement_openraft_append(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: AppendRequest,
) -> Result<AppendReply, RobustMQError> {
    let resquest_data = AppendRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::OpenRaft,
        PlacementCenterInterface::Append,
        client_pool,
        addrs,
        resquest_data,
    ).await {
        Ok(data) => match AppendReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(RobustMQError::CommonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn placement_openraft_snapshot(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: SnapshotRequest,
) -> Result<SnapshotReply, RobustMQError> {
    let request_data = SnapshotRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::OpenRaft,
        PlacementCenterInterface::Snapshot,
        client_pool,
        addrs,
        request_data,
    ).await {
        Ok(data) => match SnapshotReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(RobustMQError::CommonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}