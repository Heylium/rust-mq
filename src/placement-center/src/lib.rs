
use tokio::{
    signal,
    sync::{broadcast, mpsc},
};
use common_base::config::placement_center::placement_center_conf;
use crate::raft::apply::RaftMessage;
use crate::raft::peer::PeerMessage;

pub mod server;
pub mod raft;
mod storage;

pub async fn start_server(stop_sx: broadcast::Sender<bool>) {
    let config = placement_center_conf();
    let (raft_message_send, raft_message_recv) = mpsc::channel::<RaftMessage>(1000);
    let (peer_message_send, _) = mpsc::channel::<PeerMessage>(1000);
}