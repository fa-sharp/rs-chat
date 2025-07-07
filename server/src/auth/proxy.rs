use rocket::{
    fairing::AdHoc,
    figment::{
        providers::{Env, Format, Toml},
        Figment,
    },
    http::HeaderMap,
};
use serde::Deserialize;

/// Proxy header configuration
#[derive(Debug, Deserialize)]
pub struct ProxyHeaderConfig {
    pub proxy_username_header: String,
    pub proxy_name_header: Option<String>,
    pub proxy_logout_url: Option<String>,
}

/// Proxy user derived from headers
pub struct ProxyUser<'r> {
    username: &'r str,
    name: Option<&'r str>,
}

/// Fairing that sets up proxy header authentication, if relevant environment variables are present
pub fn setup_proxy_auth() -> AdHoc {
    AdHoc::on_ignite("Proxy header auth", |rocket| async {
        match get_config_provider().extract::<ProxyHeaderConfig>() {
            Ok(config) => {
                rocket::info!("Proxy header auth: configured");
                rocket.manage(config)
            }
            Err(_) => {
                rocket::debug!("Proxy header auth: configuration not found");
                rocket
            }
        }
    })
}

/// Read the proxy user from the given headers
pub fn get_proxy_user_from_headers<'r>(
    config: &ProxyHeaderConfig,
    headers: &'r HeaderMap,
) -> Option<ProxyUser<'r>> {
    headers
        .get_one(&config.proxy_username_header)
        .map(|username| ProxyUser {
            username,
            name: config
                .proxy_name_header
                .as_ref()
                .and_then(|name_header| headers.get_one(&name_header)),
        })
}

/// Builds and returns a Figment configuration provider that merges variables from:
/// 1. Rocket.toml file
/// 1. Environment variables prefixed with `RS_CHAT_`. In debug/dev mode, will also load
/// variables from local `.env` file
fn get_config_provider() -> Figment {
    #[cfg(debug_assertions)]
    if let Err(e) = dotenvy::dotenv() {
        println!("Failed to read .env file: {}", e);
    }

    Figment::new()
        .merge(Toml::file("Rocket.toml").nested())
        .merge(Env::prefixed("RS_CHAT_").global())
}
