use axum::extract::State;
use crate::server::http::server::HttpServerState;

pub async fn index(State(_): State<HttpServerState>) -> String {

}