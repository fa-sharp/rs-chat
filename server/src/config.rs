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
    /// Postgres Database URL
    pub database_url: String,
    /// GitHub OAuth Client ID
    pub github_client_id: String,
    /// GitHub OAuth Client Secret
    pub github_client_secret: String,
}

/// Get the server configuration variables from Rocket
pub fn get_app_config(rocket: &Rocket<Build>) -> &AppConfig {
    rocket
        .state::<AppConfig>()
        .expect("Server configuration not loaded")
}

/// Builds and returns a Figment configuration provider that merges settings from:
/// 1. Default Rocket config
/// 2. Rocket.toml file
/// 3. Environment variables prefixed with `CHAT_RS_`. In debug/dev mode, will load
/// variables from local `.env` file
pub fn get_config_provider() -> Figment {
    #[cfg(debug_assertions)]
    if let Err(e) = dotenvy::dotenv() {
        println!("Failed to read .env file: {}", e);
    }

    Figment::from(rocket::Config::default())
        .merge(Toml::file("Rocket.toml").nested())
        .merge(Env::prefixed("CHAT_RS_").global())
}
