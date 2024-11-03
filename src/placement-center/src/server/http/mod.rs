pub mod server;
pub mod index;
pub mod openraft;

pub(crate) fn v1_path(path: &str) -> String {
    format!("/v1{}", path)
}

pub(crate) fn path_list(path: &str) -> String {
    format!("{}/list", path)
}

pub(crate) fn path_create(path: &str) -> String {
    format!("{}/create", path)
}

pub(crate) fn path_update(path: &str) -> String {
    format!("{}/update", path)
}

pub(crate) fn path_delete(path: &str) -> String {
    format!("{}/delete", path)
}