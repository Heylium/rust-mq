
//<editor-fold desc="Raft">
pub fn key_name_by_first_index() -> String {
    "/raft/first_index".to_string()
}

pub fn key_name_by_last_index() -> String {
    "/raft/last_index".to_string()
}

pub fn key_name_by_hard_state() -> String {
    "/raft/hard_state".to_string()
}

pub fn key_name_by_conf_state() -> String {
    "/raft/conf_state".to_string()
}

pub fn key_name_by_entry(idx: u64) -> String {
    format!("/raft/entry/{}", idx)
}

pub fn key_name_uncommit() -> String {
    "/raft/uncommit_index".to_string()
}

pub fn key_name_snapshot() -> String {
    "/raft/snapshot".to_string()
}
//</editor-fold>


//<editor-fold desc="Cluster">
pub fn key_cluster(cluster_type: &String, cluster_name: &String) -> String {
    format!("/clusters/{}/{}", cluster_type, cluster_name)
}

pub fn key_cluster_prefix() -> String {
    "/clusters/".to_string()
}

pub fn key_cluster_prefix_by_type(cluster_type: &String) -> String {
    format!("/clusters/{}", cluster_type)
}

pub fn key_node(cluster_name: &String, node_id: u64) -> String {
    format!("/clusters/node/{}/{}", cluster_name, node_id)
}

pub fn key_node_prefix(cluster_name: &String) -> String {
    format!("/clusters/node/{}", cluster_name)
}

pub fn key_node_prefix_all() -> String {
    "/clusters/node/".to_string()
}

pub fn key_resource_config(cluster_name: String, resource_key: String) -> String {
    format!("/config/{}/{}", cluster_name, resource_key)
}

pub fn key_resource_idempotent(cluster_name: &String, produce_id: &String, seq_num: u64) -> String {
    format!("/idempotent/{}/{}/{}", cluster_name, produce_id, seq_num)
}
//</editor-fold>


//<editor-fold desc="Journal">
#[allow(dead_code)]
pub fn key_shard_prefix(cluster_name: &String) -> String {
    format!("/journal/shard/{}", cluster_name)
}

pub fn key_segment(cluster_name: &String, shard_name: &String, segment_seq: u64) -> String {
    format!(
        "/journal/segment/{}/{}/{}",
        cluster_name, shard_name, segment_seq
    )
}

#[allow(dead_code)]
pub fn key_segment_cluster_prefix(cluster_name: &String) -> String {
    format!("/journal/segment/{}", cluster_name)
}

#[allow(dead_code)]
pub fn key_segment_shard_prefix(cluster_name: &String, shard_name: &String) -> String {
    format!("/journal/segment/{}/{}", cluster_name, shard_name)
}

/** ===========MQTT========== */
pub fn storage_key_mqtt_user(cluster_name: &String, user_name: &String) -> String {
    format!("/mqtt/user/{}/{}", cluster_name, user_name)
}

pub fn storage_key_mqtt_user_cluster_prefix(cluster_name: &String) -> String {
    format!("/mqtt/user/{}", cluster_name)
}

pub fn storage_key_mqtt_topic(cluster_name: &String, user_name: &String) -> String {
    format!("/mqtt/topic/{}/{}", cluster_name, user_name)
}

pub fn storage_key_mqtt_topic_cluster_prefix(cluster_name: &String) -> String {
    format!("/mqtt/topic/{}", cluster_name)
}

pub fn storage_key_mqtt_session(cluster_name: &String, client_id: &String) -> String {
    format!("/mqtt/session/{}/{}", cluster_name, client_id)
}

pub fn storage_key_mqtt_session_cluster_prefix(cluster_name: &String) -> String {
    format!("/mqtt/session/{}", cluster_name)
}

pub fn storage_key_mqtt_last_will(cluster_name: &String, client_id: &String) -> String {
    format!("/mqtt/lastwill/{}/{}", cluster_name, client_id)
}
pub fn storage_key_mqtt_last_will_prefix(cluster_name: &String) -> String {
    format!("/mqtt/lastwill/{}", cluster_name)
}

pub fn storage_key_mqtt_node_sub_group_leader(cluster_name: &String) -> String {
    format!("/mqtt/sub_group_leader/{}", cluster_name)
}

pub fn storage_key_mqtt_acl(
    cluster_name: &String,
    resource_type: &String,
    resource_name: &String,
) -> String {
    format!(
        "/mqtt/acl/{}/{}/{}",
        cluster_name, resource_type, resource_name
    )
}

pub fn storage_key_mqtt_acl_prefix(cluster_name: &String) -> String {
    format!("/mqtt/acl/{}", cluster_name)
}

pub fn storage_key_mqtt_blacklist(
    cluster_name: &String,
    black_list_type: &String,
    resource_name: &String,
) -> String {
    format!(
        "/mqtt/blacklist/{}/{}/{}",
        cluster_name, black_list_type, resource_name
    )
}

pub fn storage_key_mqtt_blacklist_prefix(cluster_name: &String) -> String {
    format!("/mqtt/blacklist/{}", cluster_name)
}
//</editor-fold>