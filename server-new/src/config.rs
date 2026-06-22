use std::net::{IpAddr, Ipv4Addr};

use anyhow::Context;
use axum_plugin::AdHocPlugin;
use serde::Deserialize;

use crate::state::AppState;

/// Parsed app configuration
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    // Server config
    #[serde(default = "default_host")]
    pub host: IpAddr,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_request_id_header")]
    pub request_id_header: String,

    // Auth
    pub cookie_key: String,
    #[serde(default = "default_cookie_name")]
    pub cookie_name: String,
    #[serde(default = "default_session_length")]
    pub session_length: i64,

    // Security
    #[serde(default = "default_body_limit")]
    pub body_limit: usize,
    #[serde(default = "default_req_timeout")]
    pub request_timeout: u64,
}
fn default_host() -> IpAddr {
    IpAddr::V4(Ipv4Addr::LOCALHOST)
}
fn default_port() -> u16 {
    8080
}
fn default_log_level() -> String {
    "info".to_string()
}
fn default_request_id_header() -> String {
    "x-request-id".to_string()
}
fn default_cookie_name() -> String {
    "auth-rs-chat".to_string()
}
fn default_session_length() -> i64 {
    60 * 60 * 24 * 7 // 1 week
}

fn default_body_limit() -> usize {
    2 * 1024 * 1024 // 2 MB
}
fn default_req_timeout() -> u64 {
    120 // 2 minutes
}

/// Plugin that reads and validates configuration, and adds it to server state
pub fn plugin() -> AdHocPlugin<AppState> {
    AdHocPlugin::named("Config").on_init(async |mut state| {
        let config = extract_config()?;
        state.insert(config);
        Ok(state)
    })
}

/// Extract the configuration from env variables prefixed with `RS_CHAT_`.
fn extract_config() -> anyhow::Result<AppConfig> {
    let config = figment::Figment::new()
        .merge(figment::providers::Env::prefixed("RS_CHAT_"))
        .extract::<AppConfig>()
        .context("Failed to extract valid configuration")?;

    Ok(config)
}
