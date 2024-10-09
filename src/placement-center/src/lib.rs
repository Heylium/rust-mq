
use tokio::{
    signal,
    sync::{broadcast, mpsc},
};
use common_base::config::placement_center::placement_center_conf;
use crate::raft::apply::RaftMessage;

pub mod server;
pub mod raft;

pub async fn start_server(stop_sx: broadcast::Sender<bool>) {
    let config = placement_center_conf();
    let (raft_message_send, raft_message_recv) = mpsc::channel::<RaftMessage>(1000);
}