
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RobustMqError {
    #[error("io error")]
    IOJsonError(#[from] io::Error),

    #[error("{0}")]
    CommonError(String),

    // #[error("{0}")]
    // RocksdbError(#[from] rocksdb::Error),

    #[error("No available nodes in the cluster")]
    ClusterNoAvailableNode,

    #[error("{0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("Description The interface {0} submitted logs to the commit log")]
    RaftLogCommitTimeout(String),
}