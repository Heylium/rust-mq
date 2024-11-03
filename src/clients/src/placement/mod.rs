use std::sync::Arc;
use std::time::Duration;
use log::error;
use tokio::time::sleep;
use common_base::errors::RobustMQError;
use crate::placement::kv::kv_interface_call;
use crate::placement::openraft::openraft_interface_call;
use crate::poll::ClientPool;
use crate::{retry_sleep_time, retry_times};

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
                kv_interface_call(
                    interface.clone(),
                    client_poll.clone(),
                    addr.clone(),
                    request.clone(),
                ).await
            }
            PlacementCenterService::OpenRaft => {
                openraft_interface_call(
                    interface.clone(),
                    client_poll.clone(),
                    addr.clone(),
                    request.clone(),
                ).await
            }
        };

        match result {
            Ok(data) => {
                return Ok(data)
            },
            Err(e) => {
                error!(
                    "{:?}@{:?}@{},{},",
                    service.clone(),
                    interface.clone(),
                    addr.clone(),
                    e
                );
                if times > retry_times() {
                    return Err(e);
                }
                times += 1;
            }
        }
        sleep(Duration::from_secs(retry_sleep_time(times))).await;
    }
}