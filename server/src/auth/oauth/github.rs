use rocket::{get, http::CookieJar, response::Redirect, routes, Route, State};
use rocket_flex_session::Session;
use rocket_oauth2::{OAuth2, StaticProvider, TokenResponse};
use serde::Deserialize;

use crate::{
    db::{
        models::{ChatRsUser, NewChatRsUser, UpdateChatRsUser},
        services::user::UserDbService,
        DbConnection,
    },
    errors::ApiError,
};

use super::{generic_login, generic_login_callback, ChatRsAuthSession, OAuthProvider, UserData};

/// GitHub OAuth provider
pub struct GitHubProvider;

#[derive(Debug, Deserialize)]
pub struct GitHubOAuthConfig {
    github_client_id: String,
    github_client_secret: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubUserInfo {
    id: u64,
    login: String,
    name: Option<String>,
    avatar_url: Option<String>,
}

impl OAuthProvider for GitHubProvider {
    type Config = GitHubOAuthConfig;
    type UserInfo = GitHubUserInfo;

    const PROVIDER_NAME: &'static str = "GitHub";

    fn get_static_provider(_config: &Self::Config) -> StaticProvider {
        StaticProvider::GitHub
    }

    fn get_scopes(_config: Option<&Self::Config>) -> Vec<&str> {
        vec!["user:read"]
    }

    fn get_routes() -> Vec<Route> {
        routes![github_login, github_login_callback]
    }

    fn get_user_info_url(_config: &Self::Config) -> &str {
        "https://api.github.com/user"
    }

    fn get_client_id(config: &Self::Config) -> String {
        config.github_client_id.clone()
    }

    fn get_client_secret(config: &Self::Config) -> String {
        config.github_client_secret.clone()
    }

    fn create_request_headers() -> Vec<(&'static str, &'static str)> {
        vec![
            ("Accept", "application/vnd.github+json"),
            ("User-Agent", "fa-sharp/rs-chat"),
        ]
    }

    fn extract_user_data(user_info: Self::UserInfo) -> UserData {
        UserData {
            id: user_info.id.to_string(),
            name: user_info.name.unwrap_or_else(|| user_info.login),
            avatar_url: user_info.avatar_url,
        }
    }

    async fn find_linked_user(
        db: &mut UserDbService<'_>,
        user_data: &UserData,
    ) -> Result<Option<ChatRsUser>, ApiError> {
        Ok(db.find_by_github_id(&user_data.id).await?)
    }

    fn is_user_linked(user: &ChatRsUser) -> bool {
        user.github_id.is_some()
    }

    fn create_update_user(user_data: &UserData) -> UpdateChatRsUser {
        UpdateChatRsUser {
            github_id: Some(&user_data.id),
            ..Default::default()
        }
    }

    fn create_new_user(user_data: &UserData) -> NewChatRsUser {
        NewChatRsUser {
            github_id: Some(&user_data.id),
            name: &user_data.name,
            avatar_url: user_data.avatar_url.as_deref(),
            ..Default::default()
        }
    }
}

#[get("/login/github")]
async fn github_login(
    oauth2: OAuth2<GitHubUserInfo>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, ApiError> {
    generic_login::<GitHubProvider>(oauth2, cookies, None, None)
}

#[get("/login/github/callback")]
async fn github_login_callback(
    db: DbConnection,
    token: TokenResponse<GitHubUserInfo>,
    config: &State<GitHubOAuthConfig>,
    session: Session<'_, ChatRsAuthSession>,
) -> Result<Redirect, ApiError> {
    generic_login_callback::<GitHubProvider>(db, token, config.inner(), session).await
}
