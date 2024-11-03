use dashmap::DashMap;
use mobc::Pool;
use crate::placement::kv::KvServiceManager;

pub struct ClientPool {
    max_open_connection: u64,
    /// placement center
    placement_center_kv_service_pools: DashMap<String, Pool<KvServiceManager>>,
    placement_center_openraft_service_pools: DashMap<String, Pool<OpenRaftServiceManager>>,
}