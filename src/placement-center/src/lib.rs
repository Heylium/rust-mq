use std::sync::{Arc, RwLock};
use log::info;
use tokio::{
    signal,
    sync::{broadcast, mpsc},
};
use tokio::sync::broadcast::error::SendError;
use clients::poll::ClientPool;
use common_base::config::placement_center::placement_center_conf;
use crate::openraft::raft_node::{create_raft_node, start_openraft_node};
use crate::raft::apply::{RaftMachineApply, RaftMessage};
use crate::raft::machine::RaftMachine;
use crate::raft::metadata::RaftGroupMetadata;
use crate::raft::peer::PeerMessage;
use crate::raft::route::DataRoute;
use crate::server::grpc::server::start_grpc_server;
use crate::server::http::server::{start_http_server, HttpServerState};
use crate::storage::raft::RaftMachineStorage;
use crate::storage::rocksdb::RocksDBEngine;

pub mod server;
pub mod raft;
pub mod storage;
pub mod openraft;
mod requests;

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

    let client_poll = Arc::new(ClientPool::new(3));

    let (openraft_node, kvs) = create_raft_node(client_poll.clone()).await;

    let raw_stop_sx = stop_sx.clone();
    let tmp_openraft_node = openraft_node.clone();
    tokio::spawn(async move {
        start_grpc_server(
            client_poll,
            tmp_openraft_node,
            placement_center_storage,
            rocksdb_engine_handler,
            placement_cache,
            raw_stop_sx,
        )
            .await;
    });

    let tmp_openraft_node = openraft_node.clone();
    tokio::spawn(async move {
        start_openraft_node(openraft_node).await;
    });

    let raw_stop_sx = stop_sx.clone();
    tokio::spawn(async move {
        let state = HttpServerState::new(tmp_openraft_node, kvs);
        start_http_server(state, raw_stop_sx).await;
    });

    awaiting_stop(stop_sx.clone()).await;
}

pub async fn awaiting_stop(stop_send: broadcast::Sender<bool>) {
    signal::ctrl_c().await.expect("failed to listen for event");
    match stop_send.send(true) {
        Ok(_) => {
            info!(
                "{}",
                "When ctrl + c is received, the service starts to stop"
            );
        }
        Err(e) => {
            panic!("{}", e);
        }
    }
}