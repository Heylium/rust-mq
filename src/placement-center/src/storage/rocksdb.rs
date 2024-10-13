use std::collections::HashMap;
use std::fmt::format;
use std::path::Path;
use rocksdb::{ColumnFamily, DBCompactionStyle, Error, Options, SliceTransform, DB};
use log::error;
use serde::de::DeserializeOwned;
use serde::Serialize;
use common_base::config::placement_center::PlacementCenterConfig;
use common_base::errors::RobustMQError;

pub const DB_COLUMN_FAMILY_CLUSTER: &str = "cluster";

fn column_family_list() -> Vec<String> {
    let mut list = Vec::new();
    list.push(DB_COLUMN_FAMILY_CLUSTER.to_string());
    list
}

pub struct RocksDBEngine {
    pub db: DB
}

impl RocksDBEngine {
    pub fn new(config: &PlacementCenterConfig) -> Self {
        let opts: Options = Self::open_db_opts();
        let db_path = format!("{}/{}", config.data_path, "_storage_rocksdb");

        // init RocksDB
        if !Path::new(&db_path).exists() {
            DB::open(&opts, db_path.clone()).unwrap();
        }

        let cf_list = rocksdb::DB::list_cf(&opts, &db_path).unwrap();
        let mut instance = DB::open_cf(&opts, db_path.clone(), &cf_list).unwrap();

        for family in column_family_list().iter() {
            if cf_list.iter().find(|cf| cf == &family).is_none() {
                match instance.create_cf(&family, &opts) {
                    Ok(()) => {}
                    Err(err) => panic!("{}", err),
                }
            }
        }

        RocksDBEngine {
            db: instance,
        }
    }

    /// Write the data serialization to RocksDB
    pub fn write<T: Serialize + std::fmt::Debug>(&self, cf: &ColumnFamily, key: &str, value: &T) -> Result<(), String> {
        match serde_json::to_string(&value) {
            Ok(serialized) => self
                .db
                .put_cf(cf, key, serialized.into_bytes())
                .map_err(|err: Error| format!("Failed to put to ColumnFamily:{:?}", err)),
            Err(err) => Err(format!(
                "Failed to serialize to String. T: {:?}, err: {:?}",
                value, err
            )),
        }
    }

    /// Write the data serialization to RocksDB
    pub fn write_str(&self, cf: &ColumnFamily, key: &str, value: String) -> Result<(), String> {
        self.db
            .put_cf(cf, key, value.into_bytes())
            .map_err(|err| format!("Failed to put to ColumnFamily:{:?}", err))
    }

    /// Read data from the RocksDB
    pub fn read<T: DeserializeOwned>(&self, cf: &ColumnFamily, key: &str) -> Result<Option<T>, String> {
        match self.db.get_cf(cf, key) {
            Ok(opt) => match opt {
                None => Ok(None),
                Some(found) => match String::from_utf8(found) {
                    Ok(s) => match serde_json::from_str::<T>(&s) {
                        Ok(t) => Ok(Some(t)),
                        Err(err) => Err(format!("Failed to deserialize: {:?}", err)),
                    },
                    Err(err) => Err(format!("Failed to deserialize: {:?}", err)),
                }
            }
            Err(err) => Err(format!("Failed to get from ColumnFamily: {:?}", err)),
        }
    }

    /// Search data by prefix
    pub fn read_prefix(&self, cf: &ColumnFamily, search_key: &str) -> Vec<HashMap<String, Vec<u8>>> {
        let mut iter = self.db.raw_iterator_cf(cf);
        iter.seek(search_key);

        let mut result = Vec::new();
        while iter.valid() {
            let key = iter.key();
            let value = iter.value();

            let mut raw = HashMap::new();
            if key == None || value == None {
                continue;
            }
            let result_key = match String::from_utf8(key.unwrap().to_vec()) {
                Ok(s) => s,
                Err(_) => continue,
            };

            if !result_key.starts_with(search_key) {
                break;
            }
            raw.insert(result_key, value.unwrap().to_vec());
            result.push(raw);
            iter.next();
        }
        result
    }

    /// read data from all ColumnFamily
    pub fn read_all(&self) -> HashMap<String, Vec<HashMap<String, String>>> {
        let mut result: HashMap<String, Vec<HashMap<String, String>>> = HashMap::new();
        for family in column_family_list().iter() {
            let cf = self.get_column_family();
            result.insert(family.to_string(), self.read_all_by_cf(cf));
        }
        result
    }

    /// Read all data in a ColumnFamily
    pub fn read_all_by_cf(&self, cf: &ColumnFamily) -> Vec<HashMap<String, String>> {
        let mut iter = self.db.raw_iterator_cf(cf);
        iter.seek_to_first();

        let mut result: Vec<HashMap<String, String>> = Vec::new();
        while iter.valid() {
            if let Some(key) = iter.key() {
                if let Some(val) = iter.value() {
                    match String::from_utf8(key.to_vec()) {
                        Err(err) => {
                            error!("{}", err);
                        },
                        Ok(key) => match String::from_utf8(val.to_vec()) {
                            Err(err) => {
                                error!("{}", err)
                            },
                            Ok(da) => {
                                let mut raw: HashMap<String, String> = HashMap::new();
                                raw.insert(key, da);
                                result.push(raw);
                            },
                        }
                    }
                }
            }
            iter.next();
        }
        result
    }

    fn open_db_opts() -> Options {
        let transform = SliceTransform::create_fixed_prefix(10);
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_max_open_files(1000);
        opts.set_use_fsync(false);
        opts.set_bytes_per_sync(8388608);
        opts.optimize_for_point_lookup(1024);
        opts.set_table_cache_num_shard_bits(6);
        opts.set_max_write_buffer_number(32);
        opts.set_write_buffer_size(536870912);
        opts.set_target_file_size_base(1073741824);
        opts.set_min_write_buffer_number_to_merge(4);
        opts.set_level_zero_stop_writes_trigger(2000);
        opts.set_level_zero_slowdown_writes_trigger(0);
        opts.set_compaction_style(DBCompactionStyle::Universal);
        opts.set_disable_auto_compactions(true);
        opts.set_prefix_extractor(transform);
        opts.set_memtable_prefix_bloom_ratio(0.2);
        opts
    }

    pub fn get_column_family(&self) -> &ColumnFamily {
        self.cf_cluster()
    }

    pub fn cf_cluster(&self) -> &ColumnFamily {
        self.db.cf_handle(&DB_COLUMN_FAMILY_CLUSTER).unwrap()
    }

    pub fn delete(&self, cf: &ColumnFamily, key: &str) -> Result<(), RobustMQError> {
        Ok(self.db.delete_cf(cf, key)?)
    }

    pub fn exist(&self, cf: &ColumnFamily, key: &str) -> bool {
        self.db.key_may_exist_cf(cf, key)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;
    use serde::{Deserialize, Serialize};
    use tokio::fs::remove_dir;
    use tokio::time::sleep;
    use common_base::config::placement_center::PlacementCenterConfig;
    use crate::storage::rocksdb::RocksDBEngine;

    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    struct User {
        pub name: String,
        pub age: u32,
    }

    #[tokio::test]
    async fn multi_rocksdb_instance() {
        let mut config = PlacementCenterConfig::default();
        config.data_path = "/tmp/tmp_test".to_string();
        config.log.log_path = "/tmp/tmp_log".to_string();
        let rs_handler = Arc::new(RocksDBEngine::new(&config));
        for i in 1..100 {
            let rs = rs_handler.clone();
            tokio::spawn(async move {
                let key = format!("name2{}", i);
                let name = format!("rs_db_name_{}", i);
                let user = User {
                    name: name.clone(),
                    age: 20 + i,
                };
                let res4 = rs.write(rs.cf_cluster(), &key, &user);
                assert!(res4.is_ok());

                let res1 = rs.read::<User>(rs.cf_cluster(), &key);
                let r = res1.unwrap();
                assert!(!r.is_none());
                assert_eq!(r.unwrap().name, name);
                println!("spawn {},. key: {}", i, key);
                sleep(Duration::from_secs(5)).await;
            });
        }

        sleep(Duration::from_secs(5)).await;
    }

    #[tokio::test]
    async fn init_family() {
        let mut config = PlacementCenterConfig::default();
        config.data_path = "/tmp/tmp_test".to_string();
        config.data_path = "/tmp/tmp_test".to_string();
        let rs = RocksDBEngine::new(&config);
        let key = "name2";
        let res1 = rs.read::<User>(rs.cf_cluster(), key);
        assert!(res1.unwrap().is_none());

        let user = User {
            name: "lobo".to_string(),
            age: 18,
        };
        let res4 = rs.write(rs.cf_cluster(), key, &user);
        assert!(res4.is_ok());

        let res5 = rs.read::<User>(rs.cf_cluster(), key);
        let res5_rs = res5.unwrap().unwrap();
        assert_eq!(res5_rs.name, user.name);
        assert_eq!(res5_rs.age, user.age);

        let res6 = rs.delete(rs.cf_cluster(), key);
        assert!(res6.is_ok());

        match remove_dir(config.data_path).await {
            Ok(_) => {},
            Err(err) => {
                println!("{:?}", err)
            }
        }
    }

    #[tokio::test]
    async fn read_all() {
        let mut config = PlacementCenterConfig::default();
        config.data_path = "/tmp/tmp_test".to_string();
        config.data_path = "/tmp/tmp_test".to_string();
        let rs = RocksDBEngine::new(&config);

        let index = 66u64;
        let cf = rs.cf_cluster();

        let key = "/keys".to_string();
        let _ = rs.write(cf, &key, &index);
        let res1 = rs.read::<u64>(cf, &key).unwrap().unwrap();
        assert_eq!(index, res1);

        let result = rs.read_all_by_cf(rs.cf_cluster());
        assert!(result.len() > 0);
        for raw in result.clone() {
            assert!(raw.contains_key(&key));
            let v = raw.get(&key);
            assert_eq!(index.to_string(), v.unwrap().to_string());

            let _ = rs.write_str(cf, &key, v.unwrap().to_string());
            let res1 = rs.read::<u64>(cf, &key).unwrap().unwrap();
            assert_eq!(index, res1);
        }
    }

    #[tokio::test]
    async fn read_prefix() {
        let mut config = PlacementCenterConfig::default();
        config.data_path = "/tmp/tmp_test".to_string();
        config.data_path = "/tmp/tmp_test".to_string();
        let rs = RocksDBEngine::new(&config);
        rs.write_str(rs.cf_cluster(), "/v1/v1", "v11".to_string())
            .unwrap();
        rs.write_str(rs.cf_cluster(), "/v1/v2", "v12".to_string())
            .unwrap();
        rs.write_str(rs.cf_cluster(), "/v1/v3", "v13".to_string())
            .unwrap();
        rs.write_str(rs.cf_cluster(), "/v2/tmp_test/s1", "1".to_string())
            .unwrap();
        rs.write_str(rs.cf_cluster(), "/v2/tmp_test/s3", "2".to_string())
            .unwrap();
        rs.write_str(rs.cf_cluster(), "/v2/tmp_test/s2", "3".to_string())
            .unwrap();
        rs.write_str(rs.cf_cluster(), "/v3/tmp_test/s1", "1".to_string())
            .unwrap();
        rs.write_str(rs.cf_cluster(), "/v3/tmp_test/s3", "2".to_string())
            .unwrap();
        rs.write_str(rs.cf_cluster(), "/v4/tmp_test/s2", "3".to_string())
            .unwrap();

        let result = rs.read_prefix(rs.cf_cluster(), "/v1");
        assert_eq!(result.len(), 3);

        let result = rs.read_prefix(rs.cf_cluster(), "/v2");
        assert_eq!(result.len(), 3);

        let result = rs.read_prefix(rs.cf_cluster(), "/v3");
        assert_eq!(result.len(), 2);

        let result = rs.read_prefix(rs.cf_cluster(), "/v4");
        assert_eq!(result.len(), 1);
    }
}