use mobc::Connection;
use prost::{DecodeError, Message};
use common_base::errors::RobustMQError;
use protocol::openraft::{AppendReply, AppendRequest, SnapshotReply, SnapshotRequest, VoteReply, VoteRequest};
use crate::placement::openraft::OpenRaftServiceManager;

pub(crate) async fn inner_vote(
    mut client: Connection<OpenRaftServiceManager>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match VoteRequest::decode(request.as_ref()) {
        Ok(request) => match client.vote(request).await {
            Ok(result) => Ok(VoteReply::encode_to_vec(&result.into_inner())),
            Err(err) => Err(RobustMQError::GrpcServerStatus(err))
        }
        Err(err) => Err(RobustMQError::CommonError(err.to_string())),
    }
}

pub(crate) async fn inner_append(
    mut client: Connection<OpenRaftServiceManager>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match AppendRequest::decode(request.as_ref()) {
        Ok(request) => match client.append(request).await {
            Ok(result) => {
                Ok(AppendReply::encode_to_vec(&result.into_inner()))
            }
            Err(e) => Err(RobustMQError::GrpcServerStatus(e)),
        },
        Err(e) => {
            Err(RobustMQError::CommonError(e.to_string()))
        }
    }
}

pub(crate) async fn inner_snapshot(
    mut client: Connection<OpenRaftServiceManager>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match SnapshotRequest::decode(request.as_ref()) {
        Ok(request) => match client.snapshot(request).await {
            Ok(result) => {
                Ok(SnapshotReply::encode_to_vec(&result.into_inner()))
            }
            Err(e) => Err(RobustMQError::GrpcServerStatus(e)),
        },
        Err(e) => {
            Err(RobustMQError::CommonError(e.to_string()))
        }
    }
}