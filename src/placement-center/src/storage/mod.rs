use serde::{Deserialize, Serialize};
use common_base::tools::now_second;

pub mod rocksdb;
pub mod kv;
pub mod engine;
pub mod raft;
pub mod keys;

#[derive(Serialize, Deserialize, Debug)]
pub struct StorageDataWrap {
    pub data: Vec<u8>,
    pub create_time: u64,
}

impl StorageDataWrap {
    pub fn new(data: Vec<u8>) -> Self {
        StorageDataWrap {
            data,
            create_time: now_second(),
        }
    }
}
