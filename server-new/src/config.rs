use std::net::IpAddr;

use anyhow::Context;
use axum_plugin::AdHocPlugin;
use figment::providers::{Env, Format, Toml};
use serde::Deserialize;

use crate::{
    services::auth::oauth::{DiscordOAuthConfig, GitHubOAuthConfig, GoogleOAuthConfig},
    state::AppState,
};

/// Parsed app configuration
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub security: SecurityConfig,
    pub redis: RedisConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: IpAddr,
    pub port: u16,
    pub base_url: String,
    pub log_level: String,
    pub request_id_header: String,
    pub ip_header: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub cookie_key: String,
    pub cookie_name: String,
    pub session_length: i64,
    pub github: Option<GitHubOAuthConfig>,
    pub discord: Option<DiscordOAuthConfig>,
    pub google: Option<GoogleOAuthConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecurityConfig {
    pub body_limit: usize,
    pub request_timeout: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

/// Plugin that reads and validates configuration, and adds it to server state
pub fn plugin() -> AdHocPlugin<AppState> {
    AdHocPlugin::named("Config").on_init(async |mut state| {
        let config = extract_config()?;
        tracing::info!(log_level = config.server.log_level, "Config loaded!");
        state.insert(config);
        Ok(state)
    })
}

/// Extract configuration from config.toml, then environment overrides.
fn extract_config() -> anyhow::Result<AppConfig> {
    let config = figment::Figment::new()
        .merge(Toml::file("config.toml"))
        .merge(Env::prefixed("RS_CHAT_").split("__"))
        .extract::<AppConfig>()
        .context("Failed to extract valid configuration")?;

    Ok(config)
}
