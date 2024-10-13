use std::cmp;
use std::collections::HashMap;
use std::sync::Arc;
use bincode::{deserialize, serialize};
use log::{error, info};
use prost::Message;
use raft::prelude::{ConfState, Entry, HardState, Snapshot, SnapshotMetadata};
use raft::{RaftState, Result as RaftResult, StorageError};
use raft::Error;
use crate::storage::keys::{key_name_by_conf_state, key_name_by_entry, key_name_by_first_index, key_name_by_hard_state, key_name_by_last_index, key_name_snapshot, key_name_uncommit};
use crate::storage::rocksdb::RocksDBEngine;


pub struct RaftMachineStorage {
    pub uncommit_index: HashMap<u64, i8>,
    pub trigger_snap_unavailable: bool,
    pub snapshot_metadata: SnapshotMetadata,
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl RaftMachineStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        let uncommit_index = HashMap::new();
        let mut rc = RaftMachineStorage {
            snapshot_metadata: SnapshotMetadata::default(),
            trigger_snap_unavailable: false,
            uncommit_index,
            rocksdb_engine_handler,
        };
        rc.uncommit_index = rc.uncommit_index();
        rc.snapshot_metadata = rc.create_snapshot_metadata();
        rc
    }

    /// Save HardState information to RocksDB
    pub fn save_conf_state(&self, cs: ConfState) -> Result<(), String> {
        let key = key_name_by_conf_state();
        let value = ConfState::encode_to_vec(&cs);
        self.rocksdb_engine_handler
            .write(self.rocksdb_engine_handler.cf_cluster(), &key, &value)
    }

    /// Return RaftState
    pub fn raft_state(&self) -> RaftState {
        let shs = self.hard_state();
        let scs = self.conf_state();
        RaftState {
            hard_state: shs,
            conf_state: scs,
        }
    }

    /// Save HardState information to RocksDB
    pub fn hard_state(&self) -> HardState {
        let key = key_name_by_hard_state();
        let value = self
            .rocksdb_engine_handler
            .read::<Vec<u8>>(self.rocksdb_engine_handler.cf_cluster(), &key)
            .unwrap();
        if value == None {
            HardState::default()
        } else {
            HardState::decode(value.unwrap().as_ref()).unwrap()
        }
    }

    /// Save HardState information to RocksDB
    pub fn conf_state(&self) -> ConfState {
        let key = key_name_by_conf_state();
        let value = self
            .rocksdb_engine_handler
            .read::<Vec<u8>>(self.rocksdb_engine_handler.cf_cluster(), &key)
            .unwrap();
        if value.is_none() {
            ConfState::default()
        } else {
            ConfState::decode(value.unwrap().as_ref())
                .map_err(|e| tonic::Status::invalid_argument(e.to_string()))
                .unwrap()
        }
    }

    pub fn commit_index(&mut self, idx: u64) -> RaftResult<()> {
        let entry = self.entry_by_idx(idx);
        if entry.is_none() {
            info!("commit_to {} but the entry does not exist", idx);
        }

        println!(">> commit entry index:{}", idx);
        // update uncommit index
        self.uncommit_index.remove(&idx);
        self.save_uncommit_index();

        // update hs
        let mut hs = self.hard_state();
        hs.commit = idx;
        hs.term = entry.unwrap().get_term();
        let _ = self.save_hard_state(hs);
        Ok(())
    }

    pub fn append(&mut self, entries: &Vec<Entry>) -> RaftResult<()> {
        if entries.len() == 0 {
            return Ok(());
        }

        let entry_first_index = entries[0].index;

        let first_index = self.first_index();
        if first_index > entry_first_index {
            panic!(
                "overwrite compacted raft logs, compacted: {}, append: {}",
                first_index - 1,
                entry_first_index,
            );
        }

        let last_index = self.last_index();
        if last_index + 1 < entry_first_index {
            panic!(
                "raft logs should be continuous, last index: {}, new appended: {}",
                last_index, entry_first_index,
            );
        }

        for entry in entries {
            println!(">> save entry index:{}, value:{:?}", entry.index, entry);
            let data: Vec<u8> = Entry::encode_to_vec(&entry);
            let key = key_name_by_entry(entry.index);
            self.rocksdb_engine_handler
                .write(self.rocksdb_engine_handler.cf_cluster(), &key, &data)
                .unwrap();
            self.uncommit_index.insert(entry.index, 1);
            self.save_last_index(entry.index).unwrap();
        }

        self.save_uncommit_index();

        Ok(())
    }

    #[allow(dead_code)]
    pub fn entries(&self, low: u64, high: u64) -> Vec<Entry> {
        let mut entry_list: Vec<Entry> = Vec::new();
        for idx in low..=high {
            if let Some(sret) = self.entry_by_idx(idx) {
                entry_list.push(sret);
            }
        }
        entry_list
    }
}

impl RaftMachineStorage {
    pub fn first_index(&self) -> u64 {
        let key = key_name_by_first_index();
        match self
            .rocksdb_engine_handler
            .read::<u64>(self.rocksdb_engine_handler.cf_cluster(), &key)
        {
            Ok(value) => {
                if let Some(fi) = value {
                    fi
                } else {
                    self.snapshot_metadata.index + 1
                }
            }
            Err(e) => {
                error!("Failed to read the first index. The failure message is {}, and the current snapshot index is {}",e, self.snapshot_metadata.index);
                self.snapshot_metadata.index + 1
            }
        }
    }

    pub fn last_index(&self) -> u64 {
        let key = key_name_by_last_index();
        match self
            .rocksdb_engine_handler
            .read::<u64>(self.rocksdb_engine_handler.cf_cluster(), &key)
        {
            Ok(value) => {
                if let Some(li) = value {
                    li
                } else {
                    self.snapshot_metadata.index
                }
            }
            Err(e) => {
                error!("Failed to read the last index. The failure message is {}, and the current snapshot index is {}",e, self.snapshot_metadata.index);
                self.snapshot_metadata.index
            }
        }
    }

    pub fn entry_by_idx(&self, idx: u64) -> Option<Entry> {
        let key = key_name_by_entry(idx);
        match self
            .rocksdb_engine_handler
            .read::<Vec<u8>>(self.rocksdb_engine_handler.cf_cluster(), &key)
        {
            Ok(value) => {
                if let Some(vl) = value {
                    let et = Entry::decode(vl.as_ref())
                        .map_err(|e| tonic::Status::invalid_argument(e.to_string()))
                        .unwrap();
                    return Some(et);
                }
            }
            Err(e) => error!(
                "Failed to read entry. The failure information is {}, and the current index is {}",
                e, idx
            ),
        }
        None
    }

    pub fn save_last_index(&self, index: u64) -> Result<(), String> {
        let key = key_name_by_last_index();
        self.rocksdb_engine_handler
            .write(self.rocksdb_engine_handler.cf_cluster(), &key, &index)
    }

    pub fn save_first_index(&self, index: u64) -> Result<(), String> {
        let key = key_name_by_first_index();
        self.rocksdb_engine_handler
            .write(self.rocksdb_engine_handler.cf_cluster(), &key, &index)
    }

    pub fn save_hard_state(&self, hs: HardState) -> Result<(), String> {
        let key = key_name_by_hard_state();
        let val = HardState::encode_to_vec(&hs);
        self.rocksdb_engine_handler
            .write(self.rocksdb_engine_handler.cf_cluster(), &key, &val)
    }

    pub fn set_hard_state_commit(&self, commit: u64) -> Result<(), String> {
        let mut hs = self.hard_state();
        hs.commit = commit;
        self.save_hard_state(hs)
    }

    pub fn save_uncommit_index(&self) {
        let ui = self.uncommit_index.clone();
        let val = serialize(&ui).unwrap();
        let key = key_name_uncommit();
        let _ =
            self.rocksdb_engine_handler
                .write(self.rocksdb_engine_handler.cf_cluster(), &key, &val);
    }

    pub fn save_snapshot_data(&self, snapshot: Snapshot) {
        let val = Snapshot::encode_to_vec(&snapshot);
        let key = key_name_snapshot();
        let _ = self.rocksdb_engine_handler
                .write(self.rocksdb_engine_handler.cf_cluster(), &key, &val);
    }

    pub fn uncommit_index(&self) -> HashMap<u64, i8> {
        let key = key_name_uncommit();
        match self
            .rocksdb_engine_handler
            .read::<Vec<u8>>(self.rocksdb_engine_handler.cf_cluster(), &key)
        {
            Ok(data) => {
                if let Some(value) = data {
                    match deserialize(value.as_ref()) {
                        Ok(v) => return v,
                        Err(err) => error!("{}", err),
                    }
                }
            }
            Err(err) => error!("{}", err),
        }
        HashMap::new()
    }

    pub fn write_all(&mut self, data: &[u8]) {
        if data.len() == 0 {
            return;
        }

        match deserialize::<HashMap<String, Vec<HashMap<String, String>>>>(data) {
            Err(err) => {
                error!("Failed to parse the snapshot data during snapshot data recovery, error message :{}",err.to_string());
            },
            Ok(data) => {
                for (_, value) in data {
                    let cf = self.rocksdb_engine_handler.get_column_family();
                    for raw in value {
                        for (key, val) in &raw {
                            info!("key:{:?},val{:?}", key, val.to_string());
                            match self
                                .rocksdb_engine_handler
                                .write_str(cf, key, val.to_string())
                            {
                                Ok(_) => {}
                                Err(err) => {
                                    error!(
                                        "Error occurred during apply snapshot. Error message: {}",
                                        err
                                    );
                                }
                            }
                        }
                    }
                }
            },
        }
    }
}

impl RaftMachineStorage {
    pub fn apply_snapshot(&mut self, mut snapshot: Snapshot) -> RaftResult<()> {
        let mut meta = snapshot.take_metadata();
        let index = meta.index;

        if self.first_index() > index {
            return Err(Error::Store(StorageError::SnapshotOutOfDate));
        }

        self.snapshot_metadata = meta.clone();
        self.write_all(snapshot.data.as_ref());

        let mut hs = self.hard_state();
        hs.set_term(cmp::max(hs.term, meta.term));
        hs.set_commit(index);
        let _ = self.save_hard_state(hs);

        let _ = self.save_conf_state(meta.take_conf_state());
        Ok(())
    }

    /// Obtain the Entry based on the index ID
    pub fn snapshot(&mut self) -> Snapshot {
        self.create_snapshot();
        let key = key_name_snapshot();
        let value = self
            .rocksdb_engine_handler
            .read::<Vec<u8>>(self.rocksdb_engine_handler.cf_cluster(), &key)
            .unwrap();
        if value.is_none() {
            Snapshot::default()
        } else {
            Snapshot::decode(value.unwrap().as_ref())
                .map_err(|e| tonic::Status::invalid_argument(e.to_string()))
                .unwrap()
        }
    }

    /// Example Create a data snapshot for the current system
    pub fn create_snapshot(&mut self) {
        let mut sns = Snapshot::default();

        let meta = self.create_snapshot_metadata();
        sns.set_metadata(meta.clone());

        let all_data = self.rocksdb_engine_handler.read_all();
        sns.set_data(serialize(&all_data).unwrap());

        let _ = self.save_first_index(meta.get_index());

        self.save_snapshot_data(sns);
        self.snapshot_metadata = meta.clone();
    }

    pub fn create_snapshot_metadata(&self) -> SnapshotMetadata {
        let hard_state = self.hard_state();
        let conf_state = self.conf_state();

        let mut meta = SnapshotMetadata::default();
        meta.set_conf_state(conf_state);
        meta.set_index(hard_state.commit);
        meta.set_term(hard_state.term);
        meta
    }
}