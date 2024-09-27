use std::net::SocketAddr;
use tokio::{select, sync::broadcast};
use axum::Router;
use axum::routing::get;
use log::info;
use common_base::config::placement_center::placement_center_conf;
use crate::server::http::{path_list, v1_path};

pub const ROUTE_ROOT: &str = "/index";

#[derive(Clone)]
pub struct HttpServerState {}

impl HttpServerState {
    pub fn new() -> Self {
        Self {}
    }
}

pub async fn start_http_server(state: HttpServerState, stop_sx: broadcast::Sender<bool>) {
    let config = placement_center_conf();
    let ip: SocketAddr = match format!("0.0.0.0:{}", config.http_port).parse() {
        Ok(data) => data,
        Err(err) => panic!("{}", err),
    };
    info!("Broker HTTP Server start. port:{}", config.http_port);

}

fn route(state: HttpServerState) -> Router {
    let common = Router::new()
        .route(v1_path(&path_list(ROUTE_ROOT)), get(index))
}