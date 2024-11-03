use std::sync::{Arc, RwLock};
use log::info;
use openraft::Raft;
use tokio::select;
use tokio::sync::broadcast;
use tonic::transport::Server;
use clients::poll::ClientPool;
use common_base::config::placement_center::placement_center_conf;
use protocol::kv::kv_service_server::KvServiceServer;
use protocol::openraft::open_raft_service_server::OpenRaftServiceServer;
use protocol::placement::placement_center_service_server::PlacementCenterServiceServer;
use crate::openraft::typeconfig::TypeConfig;
use crate::raft::apply::RaftMachineApply;
use crate::raft::metadata::RaftGroupMetadata;
use crate::server::grpc::services_kv::GrpcKvServices;
use crate::server::grpc::services_openraft::GrpcOpenRaftServices;
use crate::server::grpc::services_raft::GrpcRaftServices;
use crate::storage::rocksdb::RocksDBEngine;

pub async fn start_grpc_server(
    client_poll: Arc<ClientPool>,
    raft_node: Raft<TypeConfig>,
    placement_center_storage: Arc<RaftMachineApply>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    placement_cluster: Arc<RwLock<RaftGroupMetadata>>,
    stop_sx: broadcast::Sender<bool>,
) {
    let config = placement_center_conf();
    let server = GrpcServer::new(config.grpc_port);
    server
        .start(
            client_poll,
            placement_center_storage,
            rocksdb_engine_handler,
            placement_cluster,
            stop_sx,
            raft_node,
        )
        .await;
}

pub struct GrpcServer {
    port: usize,
}

impl GrpcServer {
    pub fn new(port: usize) -> Self {
        Self {
            port,
        }
    }

    pub async fn start(
        &self,
        client_poll: Arc<ClientPool>,
        placement_center_storage: Arc<RaftMachineApply>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        placement_cluster: Arc<RwLock<RaftGroupMetadata>>,
        stop_sx: broadcast::Sender<bool>,
        raft_node: Raft<TypeConfig>,
    ) {
        let addr = format!("0.0.0.0:{}", self.port).parse().unwrap();
        info!("Broker Grpc Server start. port:{}", self.port);

        let kv_service_handler = GrpcKvServices::new(
            client_poll.clone(),
            placement_center_storage.clone(),
            rocksdb_engine_handler,
            placement_cluster,
        );
        let raft_service_handler = GrpcRaftServices::new(placement_center_storage);

        let openraft_service_handler = GrpcOpenRaftServices::new(raft_node);

        let mut stop_rx = stop_sx.subscribe();
        select! {
            val = stop_rx.recv() => {
                match val{
                    Ok(flag) => {
                        if flag {
                            info!("HTTP Server stopped successfully");

                        }
                    }
                    Err(_) => {}
                }
            },
            val =  Server::builder().add_service(KvServiceServer::new(kv_service_handler))
                                    .add_service(PlacementCenterServiceServer::new(raft_service_handler))
                                    .add_service(OpenRaftServiceServer::new(openraft_service_handler))
                                    .serve(addr)=>{
                match val{
                    Ok(()) => {
                    },
                    Err(e) => {
                        panic!("{}",e);
                    }
                }
            }
        }
    }
}