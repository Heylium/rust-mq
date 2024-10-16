use std::sync::Arc;
use common_base::errors::RobustMQError;
use crate::poll::ClientPool;

pub mod kv;

pub mod openraft;

#[derive(Clone, Debug)]
pub enum PlacementCenterService {
    Kv,
    OpenRaft,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PlacementCenterInterface {
    Set,
    Get,
    Delete,
    Exists,

    Vote,
    Append,
    Snapshot,
}

async fn retry_call(
    service: PlacementCenterService,
    interface: PlacementCenterInterface,
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    let mut times = 1usize;
    loop {
        let index = times % addrs.len();
        let addr = addrs.get(index).unwrap().clone();
        let result = match service {
            PlacementCenterService::Kv => {

            }
            PlacementCenterService::OpenRaft => {}
        };
    }
}