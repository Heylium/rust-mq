use std::sync::{Arc, RwLock};
use tokio::{
    signal,
    sync::{broadcast, mpsc},
};
use common_base::config::placement_center::placement_center_conf;
use crate::raft::apply::{RaftMachineApply, RaftMessage};
use crate::raft::machine::RaftMachine;
use crate::raft::metadata::RaftGroupMetadata;
use crate::raft::peer::PeerMessage;
use crate::raft::route::DataRoute;
use crate::storage::raft::RaftMachineStorage;
use crate::storage::rocksdb::RocksDBEngine;

pub mod server;
pub mod raft;
mod storage;

pub async fn start_server(stop_sx: broadcast::Sender<bool>) {
    let config = placement_center_conf();
    let (raft_message_send, raft_message_recv) = mpsc::channel::<RaftMessage>(1000);
    let (peer_message_send, _) = mpsc::channel::<PeerMessage>(1000);

    let placement_cache = Arc::new(RwLock::new(RaftGroupMetadata::new()));

    let placement_center_storage = Arc::new(RaftMachineApply::new(raft_message_send));
    let rocksdb_engine_handler: Arc<RocksDBEngine> = Arc::new(RocksDBEngine::new(&config));

    let raft_machine_storage = Arc::new(RwLock::new(RaftMachineStorage::new(
        rocksdb_engine_handler.clone(),
    )));

    let data_route = Arc::new(RwLock::new(DataRoute::new(rocksdb_engine_handler.clone())));

    let mut raft: RaftMachine = RaftMachine::new(
        placement_cache.clone(),
        data_route,
        peer_message_send,
        raft_message_recv,
        stop_sx.subscribe(),
        raft_machine_storage.clone(),
    );


}