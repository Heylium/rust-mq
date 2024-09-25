use crate::{
    config::placement_center::placement_center_conf,
    tools::{create_fold, file_exists, read_file},
};

pub fn init_placement_center_log() {
    let conf = placement_center_conf();
    if !file_exists(&conf.log.log_config) {

    }
}