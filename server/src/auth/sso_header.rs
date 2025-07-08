use rocket::{
    fairing::AdHoc,
    figment::{
        providers::{Env, Format, Toml},
        Figment,
    },
    http::{HeaderMap, Status},
    request::Outcome,
};
use serde::Deserialize;

use crate::db::{
    models::{ChatRsUser, NewChatRsUser},
    services::user::UserDbService,
};

/// SSO / proxy header configuration. Can be set via environment variables.
#[derive(Debug, Deserialize)]
struct SSOHeaderConfig {
    /// Whether SSO header authentication is enabled
    sso_header_enabled: bool,
    /// Header for unique, identifying username (default: `Remote-User`)
    sso_username_header: Option<String>,
    /// Header for display name (default: `Remote-Name`)
    sso_name_header: Option<String>,
    /// Header for groups the user belongs to (default: `Remote-Groups`)
    sso_groups_header: Option<String>,
    /// If set, only users in this group will be allowed to access the app
    sso_user_group: Option<String>,
    /// URL to redirect to in order to log out of the remote service
    sso_logout_url: Option<String>,
}

/// SSO header config added to Rocket state when enabled
#[derive(Debug, Deserialize)]
pub struct SSOHeaderMergedConfig {
    pub username_header: String,
    pub name_header: String,
    pub groups_header: String,
    pub user_group: Option<String>,
    pub logout_url: Option<String>,
}

/// Proxy user derived from headers
pub struct ProxyUser<'r> {
    pub username: &'r str,
    pub name: Option<&'r str>,
    pub groups: Option<Vec<&'r str>>,
}

/// Fairing that sets up SSO header authentication, if relevant environment variables are present
pub fn setup_sso_header_auth() -> AdHoc {
    AdHoc::on_ignite("SSO header auth", |rocket| async {
        match get_config_provider().extract::<SSOHeaderConfig>() {
            Ok(config) => {
                if config.sso_header_enabled {
                    let merged_config = SSOHeaderMergedConfig {
                        username_header: config
                            .sso_username_header
                            .unwrap_or("Remote-User".to_owned()),
                        name_header: config.sso_name_header.unwrap_or("Remote-Name".to_owned()),
                        groups_header: config
                            .sso_groups_header
                            .unwrap_or("Remote-Groups".to_owned()),
                        user_group: config.sso_user_group,
                        logout_url: config.sso_logout_url,
                    };
                    rocket::info!("SSO header auth: enabled! Config: {:?}", merged_config);
                    rocket.manage(merged_config)
                } else {
                    rocket
                }
            }
            Err(_) => rocket,
        }
    })
}

/// Handle login/authentication via SSO headers
pub async fn get_sso_auth_outcome<'r>(
    proxy_user: ProxyUser<'_>,
    sso_config: &SSOHeaderMergedConfig,
    db_service: &mut UserDbService<'_>,
) -> Outcome<ChatRsUser, &'r str> {
    if let Some(allowed_user_group) = &sso_config.user_group {
        if proxy_user
            .groups
            .is_none_or(|groups| !groups.iter().any(|group| group == allowed_user_group))
        {
            rocket::debug!("SSO header auth: user group not allowed");
            return Outcome::Error((Status::Unauthorized, "User group not allowed"));
        }
    }
    match db_service.find_by_proxy_username(proxy_user.username).await {
        Ok(Some(user)) => {
            rocket::debug!("SSO header auth: existing user found");
            return Outcome::Success(user);
        }
        Ok(None) => {
            rocket::debug!("SSO header auth: creating new user");
            match db_service
                .create(NewChatRsUser {
                    proxy_username: Some(proxy_user.username),
                    name: proxy_user.name.unwrap_or(proxy_user.username),
                    github_id: None,
                })
                .await
            {
                Ok(user) => {
                    return Outcome::Success(user);
                }
                Err(err) => {
                    rocket::error!("SSO header auth: database error: {}", err);
                    return Outcome::Error((Status::InternalServerError, "Server error"));
                }
            }
        }
        Err(err) => {
            rocket::error!("SSO header auth: database error: {}", err);
            return Outcome::Error((Status::InternalServerError, "Server error"));
        }
    }
}

/// Read the proxy user from the given headers
pub fn get_sso_user_from_headers<'r>(
    config: &SSOHeaderMergedConfig,
    headers: &'r HeaderMap,
) -> Option<ProxyUser<'r>> {
    headers
        .get_one(&config.username_header)
        .map(|username| ProxyUser {
            username,
            name: headers.get_one(&config.name_header),
            groups: headers
                .get_one(&config.groups_header)
                .map(|groups_str| groups_str.split(",").map(|group| group.trim()).collect()),
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
