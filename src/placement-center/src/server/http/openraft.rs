use std::collections::{BTreeMap, BTreeSet};
use axum::extract::State;
use openraft::error::Infallible;
use openraft::RaftMetrics;
use common_base::http_response::{error_response, success_response};
use crate::openraft::raft_node::Node;
use crate::openraft::route::AppRequestData;
use crate::openraft::typeconfig::TypeConfig;
use crate::server::http::server::HttpServerState;

pub async fn add_leadrner(State(state): State<HttpServerState>) -> String {
    let node_id = 3;
    let node = Node {
        rpc_addr: "127.0.0.0:7654".to_string(),
        node_id: 2,
    };
    match state.raft_node.add_learner(node_id, node, true).await {
        Ok(data) => {
            success_response(data)
        }
        Err(e) => {
            error_response(e.to_string())
        }
    }
}

pub async fn change_membership(State(state): State<HttpServerState>) -> String {
    let mut body = BTreeSet::new();
    body.insert(3);
    match state.raft_node.change_membership(body, true).await {
        Ok(data) => {
            success_response(data)
        }
        Err(e) => {
            error_response(e.to_string())
        }
    }
}

pub async fn init(State(state): State<HttpServerState>) -> String {
    let node_id = 3;
    let node = Node {
        rpc_addr: "127.0.0.0:7654".to_string(),
        node_id: 2,
    };

    let mut nodes = BTreeMap::new();
    nodes.insert(node_id, node);

    match state.raft_node.initialize(nodes).await {
        Ok(data) => {
            success_response(data)
        }
        Err(e) => {
            error_response(e.to_string())
        }
    }
}

pub async fn metrics(State(state): State<HttpServerState>) -> String {
    let metrics = state.raft_node.metrics().borrow().clone();
    let res: Result<RaftMetrics<TypeConfig>, Infallible> = Ok(metrics);
    success_response(res)
}

pub async fn set(State(state): State<HttpServerState>) -> String {
    let data = AppRequestData::Set {
        key: "k1".to_string(),
        value: "v1".to_string(),
    };
    match state.raft_node.client_write(data).await {
        Ok(data) => {
            success_response(data)
        }
        Err(e) => {
            error_response(e.to_string())
        }
    }
}

pub async fn kv_get(State(state): State<HttpServerState>) -> String {
    let kvs = state.kvs.read().await;
    let key = "k1".to_string();
    let value = kvs.get(&key);

    success_response(value)
}