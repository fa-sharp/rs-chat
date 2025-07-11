use rocket::{get, http::CookieJar, response::Redirect, routes, Route, State};
use rocket_flex_session::Session;
use rocket_oauth2::{OAuth2, StaticProvider, TokenResponse};
use serde::Deserialize;

use crate::{
    db::{
        models::{ChatRsUser, NewChatRsUser, UpdateChatRsUser},
        services::UserDbService,
        DbConnection,
    },
    errors::ApiError,
};

use super::{generic_login, generic_login_callback, ChatRsAuthSession, OAuthProvider, UserData};

/// Custom OIDC provider
pub struct OIDCProvider {
    config: OIDCConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OIDCConfig {
    oidc_client_id: serde_json::Value,
    oidc_client_secret: String,
    /// OIDC authorization endpoint
    oidc_auth_endpoint: String,
    /// OIDC token endpoint
    oidc_token_endpoint: String,
    /// OIDC userinfo endpoint
    oidc_userinfo_endpoint: String,
    /// OIDC scopes, separated by spaces (default: "openid profile")
    oidc_scopes: Option<String>,
    /// Name of the OIDC provider (default: "OIDC")
    pub oidc_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OIDCUserInfo {
    sub: String,
    name: Option<String>,
    preferred_username: Option<String>,
    picture: Option<String>,
}

impl OAuthProvider for OIDCProvider {
    type Config = OIDCConfig;
    type UserInfo = OIDCUserInfo;

    const PROVIDER_NAME: &'static str = "OIDC";

    fn new(config: &Self::Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    fn get_static_provider(&self) -> StaticProvider {
        StaticProvider {
            auth_uri: self.config.oidc_auth_endpoint.clone().into(),
            token_uri: self.config.oidc_token_endpoint.clone().into(),
        }
    }

    fn get_scopes(&self) -> Vec<&str> {
        self.config
            .oidc_scopes
            .as_ref()
            .map_or(vec!["openid", "profile"], |scopes| {
                scopes.split(' ').collect()
            })
    }

    fn get_routes() -> Vec<Route> {
        routes![oidc_login, oidc_login_callback]
    }

    fn get_user_info_url(&self) -> &str {
        &self.config.oidc_userinfo_endpoint
    }

    fn get_client_id(&self) -> String {
        self.config.oidc_client_id.to_string()
    }

    fn get_client_secret(&self) -> String {
        self.config.oidc_client_secret.clone()
    }

    fn create_request_headers() -> Vec<(&'static str, &'static str)> {
        vec![]
    }

    fn extract_user_data(user_info: Self::UserInfo) -> UserData {
        UserData {
            id: user_info.sub,
            name: user_info
                .name
                .or(user_info.preferred_username)
                .unwrap_or_default(),
            avatar_url: user_info.picture,
        }
    }

    async fn find_linked_user(
        db: &mut UserDbService<'_>,
        user_data: &UserData,
    ) -> Result<Option<ChatRsUser>, ApiError> {
        Ok(db.find_by_oidc_id(&user_data.id).await?)
    }

    fn is_user_linked(user: &ChatRsUser) -> bool {
        user.oidc_id.is_some()
    }

    fn create_update_user(user_data: &UserData) -> UpdateChatRsUser {
        UpdateChatRsUser {
            oidc_id: Some(&user_data.id),
            ..Default::default()
        }
    }

    fn create_new_user(user_data: &UserData) -> NewChatRsUser {
        NewChatRsUser {
            oidc_id: Some(&user_data.id),
            name: &user_data.name,
            avatar_url: user_data.avatar_url.as_deref(),
            ..Default::default()
        }
    }
}

#[get("/login/oidc")]
async fn oidc_login(
    oauth2: OAuth2<OIDCUserInfo>,
    cookies: &CookieJar<'_>,
    config: &State<OIDCConfig>,
) -> Result<Redirect, ApiError> {
    generic_login::<OIDCProvider>(oauth2, cookies, config, None)
}

#[get("/login/oidc/callback")]
async fn oidc_login_callback(
    db: DbConnection,
    token: TokenResponse<OIDCUserInfo>,
    config: &State<OIDCConfig>,
    session: Session<'_, ChatRsAuthSession>,
) -> Result<Redirect, ApiError> {
    generic_login_callback::<OIDCProvider>(db, token, config, session).await
}
