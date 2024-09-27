pub mod server;
mod index;

pub(crate) fn v1_path(path: &str) -> String {
    format!("/v1{}", path)
}

pub(crate) fn path_list(path: &str) -> String {
    format!("{}/list", path)
}