use std::path::Path;
use std::sync::Arc;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use openraft::{SnapshotMeta, StorageError};
use rocksdb::{ColumnFamilyDescriptor, Options, DB};
use serde::{Deserialize, Serialize};
use serde::de::Unexpected::Option;
use crate::openraft::sotre::log_store::LogStore;
use crate::openraft::sotre::state_machine_store::StateMachineStore;
use crate::openraft::typeconfig::TypeConfig;

pub mod log_store;
pub mod state_machine_store;

type StorageResult<T> = Result<T, StorageError<TypeConfig>>;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoredSnapshot {
    pub meta: SnapshotMeta<TypeConfig>,

    /// The data of the state machine at the time of this snapshot.
    pub data: Vec<u8>,
}

fn id_to_bin(id: u64) -> Vec<u8> {
    let mut buf = Vec::with_capacity(8);
    buf.write_u64::<BigEndian>(id).unwrap();
    buf
}

fn bin_to_id(buf: &[u8]) -> u64 {
    (&buf[0..8]).read_u64::<BigEndian>().unwrap()
}

pub(crate) async fn new_storage<P: AsRef<Path>>(db_path: P) -> (LogStore, StateMachineStore) {
    let mut db_opts = Options::default();
    db_opts.create_missing_column_families(true);
    db_opts.create_if_missing(true);

    let store = ColumnFamilyDescriptor::new("_raft_store", Options::default());
    let logs = ColumnFamilyDescriptor::new("_raft_logs", Options::default());

    let db = DB::open_cf_descriptors(&db_opts, db_path, vec![store, logs]).unwrap();
    let db = Arc::new(db);

    let log_store = LogStore { db: db.clone() };
    let sm_store = StateMachineStore::new(db).await.unwrap();

    (log_store, sm_store)
}