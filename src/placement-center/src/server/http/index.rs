use axum::extract::State;
use crate::server::http::server::HttpServerState;
use common_base::http_response::success_response;

pub async fn index(State(_): State<HttpServerState>) -> String {
    success_response("{}")
}