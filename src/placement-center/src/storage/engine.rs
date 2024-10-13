use std::sync::Arc;
use raft::Storage;
use serde::Serialize;
use serde_json::Value;
use common_base::errors::RobustMQError;
use crate::storage::rocksdb::{RocksDBEngine, DB_COLUMN_FAMILY_CLUSTER};
use crate::storage::StorageDataWrap;

pub fn engine_save_by_cluster<T>(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    key_name: String,
    value: T,
) -> Result<(), RobustMQError>
where
    T: Serialize,
{
    engine_save(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_CLUSTER,
        key_name,
        value,
    )
}

pub fn engine_get_by_cluster(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    key_name: String,
) -> Result<Option<StorageDataWrap>, RobustMQError> {
    engine_get(rocksdb_engine_handler, DB_COLUMN_FAMILY_CLUSTER, key_name)
}

pub fn engine_exists_by_cluster(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    key_name: String,
) -> Result<bool, RobustMQError> {
    engine_exists(rocksdb_engine_handler, DB_COLUMN_FAMILY_CLUSTER, key_name)
}

pub fn engine_delete_by_cluster(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    key_name: String,
) -> Result<(), RobustMQError> {
    engine_delete(rocksdb_engine_handler, DB_COLUMN_FAMILY_CLUSTER, key_name)
}

pub fn engine_prefix_list_by_cluster(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    prefix_key_name: String,
) -> Result<Vec<StorageDataWrap>, RobustMQError> {
    engine_prefix_list(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_CLUSTER,
        prefix_key_name,
    )
}

fn engine_save<T>(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    rocksdb_cluster: &str,
    key_name: String,
    value: T,
) -> Result<(), RobustMQError>
where
    T: Serialize,
{
    let cf = if rocksdb_cluster.to_string() == DB_COLUMN_FAMILY_CLUSTER.to_string() {
        rocksdb_engine_handler.cf_cluster()
    } else {
        return Err(RobustMQError::ClusterNoAvailableNode);
    };

    let content = match serde_json::to_vec(&value) {
        Ok(data) => data,
        Err(e) => {
            return Err(RobustMQError::CommonError(e.to_string()))
        },
    };

    let data = StorageDataWrap::new(content);
    match rocksdb_engine_handler.write(cf, &key_name, &data) {
        Ok(_) => Ok(()),
        Err(e) => Err(RobustMQError::CommonError(e))
    }
}

fn engine_get(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    rocksdb_cluster: &str,
    key_name: String,
) -> Result<Option<StorageDataWrap>, RobustMQError> {
    let cf = if rocksdb_cluster.to_string() == DB_COLUMN_FAMILY_CLUSTER {
        rocksdb_engine_handler.cf_cluster()
    } else {
        return Err(RobustMQError::ClusterNoAvailableNode);
    };

    match rocksdb_engine_handler.read::<StorageDataWrap>(cf, &key_name) {
        Ok(Some(data)) => Ok(Some(data)),
        Ok(None) => Ok(None),
        Err(err) => Err(RobustMQError::CommonError(err)),
    }
}

fn engine_delete(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    rocksdb_cluster: &str,
    key_name: String,
) -> Result<(), RobustMQError> {
    let cf = if rocksdb_cluster.to_string() == DB_COLUMN_FAMILY_CLUSTER.to_string() {
        rocksdb_engine_handler.cf_cluster()
    } else {
        return Err(RobustMQError::ClusterNoAvailableNode);
    };

    rocksdb_engine_handler.delete(cf, &key_name)
}

fn engine_exists(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    rocksdb_cluster: &str,
    key_name: String,
) -> Result<bool, RobustMQError> {
    let cf = if rocksdb_cluster.to_string() == DB_COLUMN_FAMILY_CLUSTER.to_string() {
        rocksdb_engine_handler.cf_cluster()
    } else {
        return Err(RobustMQError::ClusterNoAvailableNode);
    };

    Ok(rocksdb_engine_handler.exist(cf, &key_name))
}

fn engine_prefix_list(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    rocksdb_cluster: &str,
    prefix_key_name: String,
) -> Result<Vec<StorageDataWrap>, RobustMQError> {
    let cf = if rocksdb_cluster.to_string() == DB_COLUMN_FAMILY_CLUSTER.to_string() {
        rocksdb_engine_handler.cf_cluster()
    } else {
        return Err(RobustMQError::ClusterNoAvailableNode);
    };

    let data_list = rocksdb_engine_handler.read_prefix(cf, &prefix_key_name);
    let mut results = Vec::new();
    for raw in data_list {
        for (_, v) in raw {
            match serde_json::from_slice(v.as_ref()) {
                Ok(v) => results.push(v),
                Err(_) => continue,
            }
        }
    }

    Ok(results)
}