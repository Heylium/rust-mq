use crate::errors::RobustMqError;
use std::{
    fs,
    path::{self, Path},
    time::{SystemTime, UNIX_EPOCH},
};

pub fn create_fold(fold: &String) -> Result<(), RobustMqError> {
    if !Path::new(fold).exists() {
        fs::create_dir_all(fold)?
    }
    Ok(())
}

pub fn file_exists(path: &String) -> bool {
    Path::new(path).exists()
}

pub fn read_file(path: &String) -> Result<String, RobustMqError> {
    if !path::Path::new(path).exists() {
        return Err(RobustMqError::CommonError(format!(
            "File {} does not exist",
            path
        )));
    }
    Ok(fs::read_to_string(&path)?)
}

pub fn now_second() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
