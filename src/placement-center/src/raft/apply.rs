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

impl RaftMachineApply {
    pub fn new(raft_sender: tokio::sync::mpsc::Sender<RaftMessage>) -> Self {
        RaftMachineApply {
            raft_status_machine_sender: raft_sender,
        }
    }

    pub async fn transfer_leader(&self, node_id: u64) -> Result<(), RobustMqError> {
        let (sx, rx) = oneshot::channel::<RaftResponseMessage>();
        Ok(self
            .apply_raft_status_machine_message(
                RaftMessage::TransferLeader {
                    node_id,
                    chan: sx,
                },
                "transfer_leader".to_string(),
                rx,
            ).await?
        )
    }

    pub async fn apply_propose_message(
        &self,
        data: StorageData,
        action: String,
    ) -> Result<(), RobustMqError> {
        let (sx, rx) = oneshot::channel::<RaftResponseMessage>();
        Ok(self
            .apply_raft_status_machine_message(
                RaftMessage::Propose {
                    data: serialize(&data).unwrap(),
                    chan: sx,
                },
                action,
                rx,
            ).await?
        )
    }

    pub async fn  apply_raft_message(
        &self,
        message: raftPreludeMessage,
        action: String,
    ) -> Result<(), RobustMqError> {
        let (sx, rx) = oneshot::channel::<RaftResponseMessage>();
        Ok(self
            .apply_raft_status_machine_message(
                RaftMessage::Raft { message, chan: sx },
                action,
                rx,
            ).await?
        )
    }

    pub async fn apply_conf_raft_message(
        &self,
        change: ConfChange,
        action: String,
    ) -> Result<(), RobustMqError> {
        let (sx, rx) = oneshot::channel::<RaftResponseMessage>();
        Ok(self
            .apply_raft_status_machine_message(
                RaftMessage::ConfChange { change, chan: sx },
                action,
                rx,
            ).await?
        )
    }

    pub async fn apply_raft_status_machine_message(
        &self,
        message: RaftMessage,
        action: String,
        rx: Receiver<RaftResponseMessage>,
    ) ->  Result<(), RobustMqError> {
        let _ = self.raft_status_machine_sender.send(message).await;
        if !self.wait_recv_chan_resp(rx).await {
            return Err(RobustMqError::RaftLogCommitTimeout(action))
        }
        Ok(())
    }

    pub async fn wait_recv_chan_resp(&self, rx: Receiver<RaftResponseMessage>) -> bool {
        let res = timeout(Duration::from_secs(30), async {
            match rx.await {
                Ok(val) => {
                    return val
                },
                Err(_) => {
                    return RaftResponseMessage::Fail
                },
            }
        });
        match res.await {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}