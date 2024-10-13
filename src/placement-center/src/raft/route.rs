use std::sync::Arc;
use bincode::deserialize;
use common_base::errors::RobustMQError;
use crate::raft::apply::{StorageData, StorageDataType};
use crate::raft::kv::DataRouteKv;
use crate::storage::rocksdb::RocksDBEngine;

pub struct DataRoute {
    route_kv: DataRouteKv,
}

impl DataRoute {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> DataRoute {
        let route_kv = DataRouteKv::new(rocksdb_engine_handler.clone());
        DataRoute {
            route_kv
        }
    }

    pub fn route(&self, data: Vec<u8>) -> Result<(), RobustMQError> {
        let storage_data: StorageData = deserialize(data.as_ref()).unwrap();
        match storage_data.data_type {
            StorageDataType::KvSet => self.route_kv.set(storage_data.value),
            StorageDataType::KvDelete => self.route_kv.delete(storage_data.value),
        }
    }
}