mod discord;
mod github;
mod google;
mod oidc;

pub use discord::{DiscordOAuthConfig, DiscordProvider};
pub use github::{GitHubOAuthConfig, GitHubProvider};
pub use google::{GoogleOAuthConfig, GoogleProvider};
pub use oidc::{OIDCConfig, OIDCProvider};

use rocket::{fairing::AdHoc, figment::Figment, http::CookieJar, response::Redirect, Route};
use rocket_flex_session::Session;
use rocket_oauth2::{HyperRustlsAdapter, OAuth2, OAuthConfig, StaticProvider, TokenResponse};
use serde::Deserialize;
use std::future::Future;

use crate::{
    auth::ChatRsAuthSession,
    config::{get_app_config, get_config_provider},
    db::{
        models::{ChatRsUser, NewChatRsUser, UpdateChatRsUser},
        services::user::UserDbService,
        DbConnection,
    },
    errors::ApiError,
};

/// Common OAuth user data structure
#[derive(Debug)]
struct UserData {
    pub id: String,
    pub name: String,
    pub avatar_url: Option<String>,
}

/// Trait for OAuth providers
trait OAuthProvider {
    /// OAuth configuration to be extracted from environment and added to Rocket state
    type Config: for<'de> Deserialize<'de>;
    /// OAuth user info type from provider's API
    type UserInfo: for<'de> Deserialize<'de> + Send + 'static;

    const PROVIDER_NAME: &'static str;

    fn get_static_provider(config: &Self::Config) -> StaticProvider;
    fn get_scopes(config: Option<&Self::Config>) -> Vec<&str>;
    fn get_user_info_url(config: &Self::Config) -> &str;
    fn get_client_id(config: &Self::Config) -> String;
    fn get_client_secret(config: &Self::Config) -> String;
    fn get_routes() -> Vec<Route>;
    fn create_request_headers() -> Vec<(&'static str, &'static str)>;
    fn extract_user_data(user_info: Self::UserInfo) -> UserData;
    fn find_linked_user(
        db: &mut UserDbService,
        user_data: &UserData,
    ) -> impl Future<Output = Result<Option<ChatRsUser>, ApiError>> + Send;
    fn is_user_linked(user: &ChatRsUser) -> bool;
    fn create_update_user(user_data: &UserData) -> UpdateChatRsUser;
    fn create_new_user(user_data: &UserData) -> NewChatRsUser;
}

/// Fairing that sets up OAuth login and routes
pub fn setup_oauth(base_path: &str) -> AdHoc {
    let base_path = base_path.to_owned();
    let config_provider = get_config_provider();

    AdHoc::on_ignite("OAuth", |mut rocket| async move {
        rocket = setup_oauth_provider::<GitHubProvider>(rocket, &base_path, &config_provider);
        rocket = setup_oauth_provider::<GoogleProvider>(rocket, &base_path, &config_provider);
        rocket = setup_oauth_provider::<DiscordProvider>(rocket, &base_path, &config_provider);
        rocket = setup_oauth_provider::<OIDCProvider>(rocket, &base_path, &config_provider);

        rocket
    })
}

/// Setup the given OAuth provider on the Rocket instance, if relevant environment variables are set
fn setup_oauth_provider<P: OAuthProvider>(
    rocket: rocket::Rocket<rocket::Build>,
    base_path: &str,
    config_provider: &Figment,
) -> rocket::Rocket<rocket::Build>
where
    P::Config: Send + Sync + 'static,
    P::UserInfo: Send + Sync + 'static,
{
    if let Ok(config) = config_provider.extract::<P::Config>() {
        rocket::info!("OAuth: {} login enabled!", P::PROVIDER_NAME);

        let callback_path = format!(
            "{}/login/{}/callback",
            base_path,
            P::PROVIDER_NAME.to_lowercase()
        );
        let oauth_config = OAuthConfig::new(
            P::get_static_provider(&config),
            P::get_client_id(&config),
            P::get_client_secret(&config),
            Some(format!(
                "{}{}",
                get_app_config(&rocket).server_address,
                callback_path
            )),
        );

        rocket
            .manage(config)
            .mount(base_path, P::get_routes())
            .attach(rocket_oauth2::OAuth2::<P::UserInfo>::custom(
                HyperRustlsAdapter::default(),
                oauth_config,
            ))
    } else {
        rocket
    }
}

/// Login redirect for the OAuth provider.
fn generic_login<P: OAuthProvider>(
    oauth2: OAuth2<P::UserInfo>,
    cookies: &CookieJar<'_>,
    config: Option<&P::Config>,
    extra_params: Option<&[(&str, &str)]>,
) -> Result<Redirect, ApiError> {
    oauth2
        .get_redirect_extras(
            cookies,
            P::get_scopes(config).as_slice(),
            extra_params.unwrap_or_default(),
        )
        .map_err(|e| ApiError::Authentication(format!("Failed to get redirect: {}", e)))
}

/// Login callback for the OAuth provider:
/// 1. Validates token and fetches user info from the provider's API
/// 1. Checks if user exists in our database
/// 1. Either creates new user or logs in / links an existing user
async fn generic_login_callback<P: OAuthProvider>(
    mut db: DbConnection,
    token: TokenResponse<P::UserInfo>,
    config: &P::Config,
    mut session: Session<'_, ChatRsAuthSession>,
) -> Result<Redirect, ApiError> {
    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| ApiError::Authentication(format!("Failed to build reqwest client: {}", e)))?;
    let mut request = client
        .get(P::get_user_info_url(config))
        .header("Authorization", format!("Bearer {}", token.access_token()));
    for (key, value) in P::create_request_headers() {
        request = request.header(key, value);
    }

    let user_info: P::UserInfo = request
        .send()
        .await
        .map_err(|e| {
            ApiError::Authentication(format!("Failed to get {} user: {}", P::PROVIDER_NAME, e))
        })?
        .json()
        .await
        .map_err(|e| ApiError::Authentication(format!("Failed to deserialize response: {}", e)))?;
    let user_data = P::extract_user_data(user_info);

    let mut db_service = UserDbService::new(&mut db);
    match P::find_linked_user(&mut db_service, &user_data).await? {
        // Existing linked user found: create new session
        Some(existing_user) => {
            session.set(ChatRsAuthSession::new(existing_user.id));
        }
        None => match session.get() {
            // No linked user and no session found: create new user and session
            None => {
                let new_user = db_service.create(P::create_new_user(&user_data)).await?;
                session.set(ChatRsAuthSession::new(new_user.id));
            }
            // No linked user but there is a current session
            Some(session_data) => {
                let user = db_service
                    .find_by_id(&session_data.user_id)
                    .await?
                    .ok_or_else(|| {
                        ApiError::Authentication(format!(
                            "User not found: {}",
                            session_data.user_id
                        ))
                    })?;
                match P::is_user_linked(&user) {
                    // User is not linked: link them to this OAuth provider
                    false => {
                        db_service
                            .update(&session_data.user_id, P::create_update_user(&user_data))
                            .await?;
                    }
                    // User is already linked to this OAuth provider
                    true => {
                        return Err(ApiError::Authentication(format!(
                            "User already linked to {}",
                            P::PROVIDER_NAME
                        )));
                    }
                }
            }
        },
    }

    Ok(Redirect::to("/"))
}
