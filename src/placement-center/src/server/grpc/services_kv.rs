use std::sync::{Arc, RwLock};
use prost::Message;
use tonic::{Request, Response, Status};
use clients::placement::kv::call::{placement_delete, placement_set};
use clients::poll::ClientPool;
use common_base::errors::RobustMQError;
use protocol::kv::kv_service_server::KvService;
use protocol::kv::{CommonReply, DeleteRequest, ExistsReply, ExistsRequest, GetReply, GetRequest, SetRequest};
use crate::raft::apply::{RaftMachineApply, StorageData, StorageDataType};
use crate::raft::metadata::RaftGroupMetadata;
use crate::storage::kv::KvStorage;
use crate::storage::rocksdb::RocksDBEngine;

pub struct GrpcKvServices {
    client_poll: Arc<ClientPool>,
    placement_center_storage: Arc<RaftMachineApply>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    placement_cluster: Arc<RwLock<RaftGroupMetadata>>,
}

impl GrpcKvServices {
    pub fn new(
        client_poll: Arc<ClientPool>,
        placement_center_storage: Arc<RaftMachineApply>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        placement_cluster: Arc<RwLock<RaftGroupMetadata>>,
    ) -> Self {
        GrpcKvServices {
            client_poll,
            placement_center_storage,
            rocksdb_engine_handler,
            placement_cluster,
        }
    }

    pub fn is_leader(&self) -> bool {
        self.placement_cluster.read().unwrap().is_leader()
    }

    pub fn leader_addr(&self) -> String {
        self.placement_cluster.read().unwrap().leader_addr()
    }
}

#[tonic::async_trait]
impl KvService for GrpcKvServices {
    async fn set(&self, request: Request<SetRequest>) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();

        if req.key.is_empty() || req.value.is_empty() {
            return Err(Status::cancelled(
                RobustMQError::ParameterCannotBeNull("key or value".to_string()).to_string(),
            ));
        }

        if !self.is_leader() {
            let leader_addr = self.leader_addr();
            match placement_set(self.client_poll.clone(), vec![leader_addr], req).await {
                Ok(reply) => {
                    return Ok(Response::new(reply));
                }
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        }

        // Raft state machine is used to store Node data
        let data = StorageData::new(StorageDataType::KvSet, SetRequest::encode_to_vec(&req));
        match self
            .placement_center_storage
            .apply_propose_message(data, "set".to_string())
            .await
        {
            Ok(_) => Ok(Response::new(CommonReply::default())),
            Err(e) => {
                Err(Status::cancelled(e.to_string()))
            }
        }
    }

    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();

        if req.key.is_empty() {
            return Err(Status::cancelled(
                RobustMQError::ParameterCannotBeNull("key".to_string()).to_string(),
            ));
        }

        if !self.is_leader() {
            let leader_addr = self.leader_addr();
            match placement_delete(self.client_poll.clone(), vec![leader_addr], req).await {
                Ok(reply) => {
                    return Ok(Response::new(reply));
                }
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        }

        // Raft state machine is used to store Node data
        let data = StorageData::new(
            StorageDataType::KvDelete,
            DeleteRequest::encode_to_vec(&req),
        );
        match self
            .placement_center_storage
            .apply_propose_message(data, "delete".to_string())
            .await
        {
            Ok(_) => Ok(Response::new(CommonReply::default())),
            Err(e) => {
                Err(Status::cancelled(e.to_string()))
            }
        }
    }

    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetReply>, Status> {
        let req = request.into_inner();

        if req.key.is_empty() {
            return Err(Status::cancelled(
                RobustMQError::ParameterCannotBeNull("key".to_string()).to_string(),
            ));
        }

        let kv_storage = KvStorage::new(self.rocksdb_engine_handler.clone());
        let mut reply = GetReply::default();
        match kv_storage.get(req.key) {
            Ok(Some(data)) => {
                reply.value = data;
                return Ok(Response::new(reply));
            }
            Ok(None) => {}
            Err(e) => return Err(Status::cancelled(e.to_string())),
        }

        Ok(Response::new(reply))
    }

    async fn exists(&self, request: Request<ExistsRequest>) -> Result<Response<ExistsReply>, Status> {
        let req = request.into_inner();

        if req.key.is_empty() {
            return Err(Status::cancelled(
                RobustMQError::ParameterCannotBeNull("key".to_string()).to_string(),
            ));
        }

        let kv_storage = KvStorage::new(self.rocksdb_engine_handler.clone());
        match kv_storage.exists(req.key) {
            Ok(flag) => {
                Ok(Response::new(ExistsReply { flag }))
            }
            Err(e) => {
                Err(Status::cancelled(e.to_string()))
            }
        }
    }
}