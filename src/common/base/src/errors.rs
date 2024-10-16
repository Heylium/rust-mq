
use std::io;
use tonic::Status;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RobustMQError {
    #[error("io error")]
    IOJsonError(#[from] io::Error),

    #[error("{0}")]
    CommonError(String),

    #[error("{0}")]
    RocksdbError(#[from] rocksdb::Error),

    #[error("No available nodes in the cluster")]
    ClusterNoAvailableNode,

    #[error("{0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("Description The interface {0} submitted logs to the commit log")]
    RaftLogCommitTimeout(String),

    #[error("Grpc call of the node failed,Grpc status was {0}")]
    GrpcServerStatus(Status),
}