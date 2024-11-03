use std::sync::{Arc, RwLock};
use openraft::Raft;
use tonic::{Request, Response, Status};
use clients::poll::ClientPool;
use common_base::errors::RobustMQError;
use protocol::kv::kv_service_server::KvService;
use protocol::kv::{CommonReply, DeleteRequest, ExistsReply, ExistsRequest, GetReply, GetRequest, SetRequest};
use crate::openraft::route::AppRequestData;
use crate::openraft::typeconfig::TypeConfig;
use crate::raft::apply::RaftMachineApply;
use crate::raft::metadata::RaftGroupMetadata;
use crate::storage::kv::KvStorage;
use crate::storage::rocksdb::RocksDBEngine;

pub struct GrpcKvServices {
    client_poll: Arc<ClientPool>,
    placement_center_storage: Arc<RaftMachineApply>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    placement_cluster: Arc<RwLock<RaftGroupMetadata>>,
    raft_node: Raft<TypeConfig>,
}

impl GrpcKvServices {
    pub fn new(
        client_poll: Arc<ClientPool>,
        placement_center_storage: Arc<RaftMachineApply>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        placement_cluster: Arc<RwLock<RaftGroupMetadata>>,
        raft_node: Raft<TypeConfig>,
    ) -> Self {
        GrpcKvServices {
            client_poll,
            placement_center_storage,
            rocksdb_engine_handler,
            placement_cluster,
            raft_node,
        }
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

        let data = AppRequestData::Set {
            key: "k1".to_string(),
            value: "v1".to_string(),
        };

        match self.raft_node.client_write(data).await {
            Ok(data) => {
                Ok(Response::new(CommonReply::default()))
            }
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

        let data = AppRequestData::Delete {
            key: "k1".to_string(),
        };

        match self.raft_node.client_write(data).await {
            Ok(data) => {
                Ok(Response::new(CommonReply::default()))
            }
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