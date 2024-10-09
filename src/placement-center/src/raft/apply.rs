use bincode::serialize;
use common_base::errors::RobustMqError;
use raft::eraftpb::ConfChange;
use raft::eraftpb::Message as raftPreludeMessage;
use serde::Deserialize;
use serde::Serialize;
use tokio::sync::oneshot;
use tokio::sync::oneshot::Receiver;
use tokio::sync::oneshot::Sender;
use tokio::time::timeout;
use std::fmt;
use std::fmt::Formatter;
use std::time::Duration;

pub enum RaftResponseMessage {
    Success,
    Fail,
}

pub enum RaftMessage {
    ConfChange {
        change: ConfChange,
        chan: Sender<RaftResponseMessage>,
    },
    // Received a message from another node
    Raft {
        message: raftPreludeMessage,
        chan: Sender<RaftResponseMessage>,
    },

    TransferLeader {
        node_id: u64,
        chan: Sender<RaftResponseMessage>,
    },

    // The data sent by the client is received. Procedure
    Propose {
        data: Vec<u8>,
        chan:  Sender<RaftResponseMessage>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum StorageDataType {
    // kv
    KvSet,
    KvDelete,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StorageData {
    pub data_type: StorageDataType,
    pub value: Vec<u8>,
}

impl StorageData {
    pub fn new(data_type: StorageDataType, value: Vec<u8>) -> StorageData {
        StorageData {
            data_type,
            value,
        }
    }
}

impl fmt::Display for StorageData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({:?}, {:?})", self.data_type, self.value)
    }
}


pub struct RaftMachineApply {
    raft_status_machine_sender: tokio::sync::mpsc::Sender<RaftMessage>,
}