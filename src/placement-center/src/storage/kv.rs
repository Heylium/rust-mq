use crate::storage::engine::{engine_delete_by_cluster, engine_exists_by_cluster, engine_get_by_cluster};
use crate::storage::rocksdb::RocksDBEngine;
use common_base::errors::RobustMQError;
use std::sync::Arc;

pub struct KvStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl KvStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        KvStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn set(&self, key: String, value: String) -> Result<(), RobustMQError> {
        engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key)
    }

    pub fn get(&self, key: String) -> Result<Option<String>, RobustMQError> {
        match engine_get_by_cluster(self.rocksdb_engine_handler.clone(), key) {
            Ok(Some(data)) => match serde_json::from_slice::<String>(&data.data) {
                Ok(data) => Ok(Some(data)),
                Err(e) => Err(e.into()),
            },
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn delete(&self, key: String) -> Result<(), RobustMQError> {
        engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key)
    }

    pub fn exists(&self, key: String) -> Result<bool, RobustMQError> {
        engine_exists_by_cluster(self.rocksdb_engine_handler.clone(), key)
    }
}