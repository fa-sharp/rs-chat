use rocket::{get, http::CookieJar, response::Redirect, routes, Route};
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

/// Google OAuth provider
pub struct GoogleProvider;

#[derive(Debug, Deserialize)]
pub struct GoogleOAuthConfig {
    pub google_client_id: String,
    pub google_client_secret: String,
}

#[derive(Debug, Deserialize)]
pub struct GoogleUserInfo {
    sub: String,
    name: String,
    picture: Option<String>,
}

impl OAuthProvider for GoogleProvider {
    type Config = GoogleOAuthConfig;
    type UserInfo = GoogleUserInfo;

    const PROVIDER_NAME: &'static str = "Google";
    const STATIC_PROVIDER: StaticProvider = StaticProvider::Google;

    fn get_scopes() -> &'static [&'static str] {
        &["openid", "profile"]
    }

    fn get_routes() -> Vec<Route> {
        routes![google_login, google_login_callback]
    }

    fn get_user_info_url() -> &'static str {
        "https://www.googleapis.com/oauth2/v3/userinfo"
    }

    fn get_client_id(config: &Self::Config) -> String {
        config.google_client_id.clone()
    }

    fn get_client_secret(config: &Self::Config) -> String {
        config.google_client_secret.clone()
    }

    fn create_request_headers() -> Vec<(&'static str, &'static str)> {
        vec![]
    }

    fn extract_user_data(user_info: Self::UserInfo) -> UserData {
        UserData {
            id: user_info.sub,
            name: user_info.name,
            avatar_url: user_info.picture,
        }
    }

    async fn find_existing_user(
        db: &mut UserDbService<'_>,
        user_data: &UserData,
    ) -> Result<Option<ChatRsUser>, ApiError> {
        Ok(db.find_by_google_id(&user_data.id).await?)
    }

    fn is_user_linked(user: &ChatRsUser) -> bool {
        user.google_id.is_some()
    }

    fn create_update_user(user_data: &UserData) -> UpdateChatRsUser {
        UpdateChatRsUser {
            google_id: Some(&user_data.id),
            ..Default::default()
        }
    }

    fn create_new_user(user_data: &UserData) -> NewChatRsUser {
        NewChatRsUser {
            google_id: Some(&user_data.id),
            name: &user_data.name,
            avatar_url: user_data.avatar_url.as_deref(),
            ..Default::default()
        }
    }
}

#[get("/login/google")]
async fn google_login(
    oauth2: OAuth2<GoogleUserInfo>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, ApiError> {
    generic_login::<GoogleProvider>(oauth2, cookies).await
}

#[get("/login/google/callback")]
async fn google_login_callback(
    db: DbConnection,
    token: TokenResponse<GoogleUserInfo>,
    session: Session<'_, ChatRsAuthSession>,
) -> Result<Redirect, ApiError> {
    generic_login_callback::<GoogleProvider>(db, token, session).await
}
