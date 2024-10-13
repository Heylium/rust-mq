use log4rs::config::InitError;
use crate::{
    config::placement_center::placement_center_conf,
    tools::{create_fold, file_exists, read_file},
};
use crate::errors::RobustMQError;

pub fn init_placement_center_log() {
    let conf = placement_center_conf();
    if !file_exists(&conf.log.log_config) {
        panic!(
            "Logging configuration file {} does not exist",
            conf.log.log_config
        );
    }

    match create_fold(&conf.log.log_path) {
        Ok(()) => {}
        Err(_) => {
            panic!("Failed to initialize log directory {}", conf.log.log_path);
        }
    }

    let content = match read_file(&conf.log.log_config) {
        Ok(content) => content,
        Err(err) => panic!("{}", err.to_string()),
    };

    let config_content = content.replace("{$path}", &conf.log.log_path);
    println!("{}", "log config:");
    println!("{}", config_content);
    let config = match serde_yaml::from_str(&config_content) {
        Ok(data) => data,
        Err(err) => {
            panic!(
                "Failed to parse the contents of the config file {} with error message :{}",
                conf.log.log_config,
                err.to_string()
            );
        }
    };

    match log4rs::init_raw_config(config) {
        Ok(_) => {}
        Err(err) => {
            panic!("{}", err.to_string());
        }
    }
}