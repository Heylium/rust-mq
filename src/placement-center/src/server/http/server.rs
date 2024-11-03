use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::{select, sync::broadcast};
use axum::Router;
use axum::routing::{delete, get, post, put};
use log::info;
use openraft::Raft;
use tokio::sync::RwLock;
use common_base::config::placement_center::placement_center_conf;
use crate::openraft::typeconfig::TypeConfig;
use crate::server::http::{index::index, path_create, path_delete, path_list, path_update, v1_path};

pub const ROUTE_ROOT: &str = "/index";
pub const ROUTE_ADD_LEARNER: &str = "/add-learner";
pub const ROUTE_CHANGE_MEMBERSHIP: &str = "/change-membership";
pub const ROUTE_INIT: &str = "/init";
pub const ROUTE_METRICS: &str = "/metrics";
pub const ROUTE_SET: &str = "/set";
pub const ROUTE_GET: &str = "/get";

#[derive(Clone)]
pub struct HttpServerState {
    pub raft_node: Raft<TypeConfig>,
    pub kvs: Arc<RwLock<BTreeMap<String, String>>>,
}

impl HttpServerState {
    pub fn new(raft_node: Raft<TypeConfig>, kvs: Arc<RwLock<BTreeMap<String, String>>>) -> Self {
        Self {
            raft_node,
            kvs,
        }
    }
}

pub async fn start_http_server(state: HttpServerState, stop_sx: broadcast::Sender<bool>) {
    let config = placement_center_conf();
    let ip: SocketAddr = match format!("0.0.0.0:{}", config.http_port).parse() {
        Ok(data) => data,
        Err(err) => panic!("{}", err),
    };
    info!("Broker HTTP Server start. port:{}", config.http_port);
    let app = routes(state);

    let mut stop_rx = stop_sx.subscribe();

    let listener = match tokio::net::TcpListener::bind(ip).await {
        Ok(data) => data,
        Err(err) => {
            panic!("{}", err);
        }
    };

    select! {
        val = stop_rx.recv() => {
            match val {
                Ok(flag) => {
                    if flag {
                        info!("HTTP Server stopped successfully")
                    }
                }
                Err(_) => {}
            }
        },
        val =  axum::serve(listener, app.clone()) => {
            match val {
                Ok(()) => {},
                Err(err) => {
                    panic!("{}", err);
                }
            }
        }
    }
}

fn routes(state: HttpServerState) -> Router {
    let common = Router::new()
        .route(&v1_path(&path_list(ROUTE_ROOT)), get(index))
        .route(&v1_path(&path_create(ROUTE_ROOT)), post(index))
        .route(&v1_path(&path_update(ROUTE_ROOT)), put(index))
        .route(&v1_path(&path_delete(ROUTE_ROOT)), delete(index));

    let app = Router::new().merge(common);
    app.with_state(state)
}