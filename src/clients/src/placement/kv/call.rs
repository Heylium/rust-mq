use std::sync::Arc;
use prost::Message;
use common_base::errors::RobustMQError;
use protocol::kv::{CommonReply, SetRequest};
use crate::poll::ClientPool;

pub async fn placement_set(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: SetRequest,
) -> Result<CommonReply, RobustMQError> {
    let request_data = SetRequest::encode_to_vec(&request);
    match ret {  }
}