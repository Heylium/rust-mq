use std::sync::Arc;
use prost::Message;
use common_base::errors::RobustMQError;
use protocol::kv::{DeleteRequest, SetRequest};
use crate::storage::kv::KvStorage;
use crate::storage::rocksdb::RocksDBEngine;

pub struct DataRouteKv {
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    kv_storage: KvStorage,
}

impl DataRouteKv {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        let kv_storage = KvStorage::new(rocksdb_engine_handler.clone());
        DataRouteKv {
            rocksdb_engine_handler,
            kv_storage,
        }
    }

    pub fn set(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req: SetRequest = SetRequest::decode(value.as_ref()).unwrap();
        self.kv_storage.set(req.key, req.value)
    }

    pub fn delete(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req: DeleteRequest = DeleteRequest::decode(value.as_ref()).unwrap();
        self.kv_storage.delete(req.key)
    }
}