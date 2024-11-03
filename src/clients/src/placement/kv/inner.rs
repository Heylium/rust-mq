use crate::placement::kv::KvServiceManager;
use common_base::errors::RobustMQError;
use mobc::Connection;
use prost::{DecodeError, Message};
use protocol::kv::{CommonReply, DeleteRequest, ExistsReply, ExistsRequest, GetReply, GetRequest, SetRequest};

pub(crate) async fn inner_get(
    mut client: Connection<KvServiceManager>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match GetRequest::decode(request.as_ref()) {
        Ok(request) => match client.get(request).await {
            Ok(result) => Ok(GetReply::encode_to_vec(&result.into_inner())),
            Err(e) => Err(RobustMQError::GrpcServerStatus(e))
        },
        Err(e) => Err(RobustMQError::CommonError(e.to_string())),
    }
}

pub(crate) async fn inner_set(
    mut client: Connection<KvServiceManager>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match SetRequest::decode(request.as_ref()) {
        Ok(request) => match client.set(request).await {
            Ok(result) => Ok(CommonReply::encode_to_vec(&result.into_inner())), // Set operation doesn't return a response
            Err(e) => Err(RobustMQError::GrpcServerStatus(e))
        },
        Err(e) => Err(RobustMQError::CommonError(e.to_string())),
    }
}

pub(crate) async fn inner_delete(
    mut client: Connection<KvServiceManager>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match DeleteRequest::decode(request.as_ref()) {
        Ok(request) => match client.delete(request).await {
            Ok(result) => Ok(CommonReply::encode_to_vec(&result.into_inner())),
            Err(e) => Err(RobustMQError::GrpcServerStatus(e))
        },
        Err(e) => Err(RobustMQError::CommonError(e.to_string())),
    }
}

pub(crate) async fn inner_exists(
    mut client: Connection<KvServiceManager>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match ExistsRequest::decode(request.as_ref()) {
        Ok(request) => match client.exists(request).await {
            Ok(result) => Ok(ExistsReply::encode_to_vec(&result.into_inner())),
            Err(e) => Err(RobustMQError::GrpcServerStatus(e)),
        },
        Err(e) => Err(RobustMQError::CommonError(e.to_string()))
    }
}