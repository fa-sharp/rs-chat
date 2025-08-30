use rocket::{
    figment::{
        providers::{Env, Format, Toml},
        Figment,
    },
    Build, Rocket,
};
use serde::{Deserialize, Serialize};

/// Main server config (settings are merged with Rocket's default config)
#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    /// 32-byte hex string (64 characters) used for encrypting cookies and API keys
    pub secret_key: String,
    /// Server address, used for OAuth redirects(e.g. "http://localhost:8000" or "https://example.com")
    pub server_address: String,
    /// Static files directory (default: "../web/dist")
    pub static_path: Option<String>,
    /// Local data directory (default: "/data")
    pub data_dir: Option<String>,
    /// Postgres Database URL
    pub database_url: String,
    /// Redis connection URL
    pub redis_url: String,
    /// Redis static pool size (default: 4)
    pub redis_pool: Option<usize>,
    /// Maximum number of concurrent Redis connections for streaming (default: 20)
    pub max_streams: Option<usize>,
}

/// Get the server configuration variables from Rocket
pub fn get_app_config(rocket: &Rocket<Build>) -> &AppConfig {
    rocket
        .state::<AppConfig>()
        .expect("Environment variables missing!")
}

/// Builds and returns a Figment configuration provider that merges settings from:
/// 1. Default Rocket config
/// 2. Rocket.toml file
/// 3. Environment variables prefixed with `RS_CHAT_`. In debug/dev mode, will also load
/// variables from local `.env` file
pub fn get_config_provider() -> Figment {
    #[cfg(debug_assertions)]
    if let Err(e) = dotenvy::dotenv() {
        println!("Failed to read .env file: {}", e);
    }

    Figment::from(rocket::Config::default())
        .merge(Toml::file("Rocket.toml").nested())
        .merge(Env::prefixed("RS_CHAT_").global())
}
