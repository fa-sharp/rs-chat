use std::{net::IpAddr, path::PathBuf};

use axum_plugin::figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};

use crate::services::auth::{
    oauth::{DiscordOAuthConfig, GitHubOAuthConfig, GoogleOAuthConfig, OidcConfig},
    proxy::ProxyHeaderConfig,
};

/// Extract configuration from defaults, local `config.toml`, then `RS_CHAT_` environment variables split by `__`.
/// See https://docs.rs/figment/latest/figment/index.html#for-application-authors
pub fn figment() -> Figment {
    Figment::from(Serialized::defaults(AppConfig::default()))
        .merge(Toml::file("config.toml"))
        .merge(Env::prefixed("RS_CHAT_").split("__"))
}

/// Parsed app configuration
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub services: ServiceConfig,
    pub security: SecurityConfig,
    pub redis: RedisConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: IpAddr,
    pub port: u16,
    pub base_url: String,
    pub log_level: String,
    pub data_dir: PathBuf,
    pub web_root: String,
    pub request_id_header: String,
    pub ip_header: Option<String>,
}
impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
            port: 8080,
            base_url: String::from("http://localhost:8080"),
            log_level: String::from("info"),
            data_dir: PathBuf::from("/data"),
            web_root: String::from("../web/dist"),
            request_id_header: String::from("x-request-id"),
            ip_header: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}
impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: String::from("postgres://localhost:5432"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub streamer_url: String,
    pub streamer_api_key: String,
}
impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            streamer_url: String::from("http://localhost:8081"),
            streamer_api_key: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub encryption_key: String,
    pub cookie_name: String,
    pub session_length: i64,
    pub github: Option<GitHubOAuthConfig>,
    pub discord: Option<DiscordOAuthConfig>,
    pub google: Option<GoogleOAuthConfig>,
    pub oidc: Option<OidcConfig>,
    pub proxy: ProxyHeaderConfig,
}
impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            encryption_key: String::new(),
            cookie_name: String::from("auth-rs-chat"),
            session_length: 604800, // 1 week in seconds
            github: None,
            discord: None,
            google: None,
            oidc: None,
            proxy: ProxyHeaderConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub body_limit: usize,
    pub upload_limit: usize,
    pub request_timeout: u64,
}
impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            body_limit: 5 * 1024 * 1024,   // 5 MB
            upload_limit: 5 * 1024 * 1024, // 5 MB
            request_timeout: 120,          // 2 minutes
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: usize,
    pub timeout: u64,
}
impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: String::from("redis://localhost:6379"),
            pool_size: 4,
            timeout: 10, // 10 seconds
        }
    }
}
