use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use simple_oauth::{
    SimpleOAuthClient, SimpleOAuthProvider,
    types::{OAuthCredentials, StandardTokenResponse, UserInfo},
};
use tower_sessions::Session;

use crate::{
    config::AppConfig,
    db::{
        DbPool, DbService,
        models::{ChatRsUser, NewChatRsUser, UpdateChatRsUser},
    },
    services::auth::{AuthError, AuthResult, UserSession},
};

mod discord;
mod github;
mod google;

pub use discord::DiscordOAuthConfig;
pub use github::GitHubOAuthConfig;
pub use google::GoogleOAuthConfig;

/// Supported OAuth providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OAuthProviderEnum {
    Github,
    Discord,
    Google,
}
impl OAuthProviderEnum {
    pub fn as_str(&self) -> &str {
        match self {
            OAuthProviderEnum::Github => "github",
            OAuthProviderEnum::Discord => "discord",
            OAuthProviderEnum::Google => "google",
        }
    }
}

/// Trait for all OAuth providers
pub trait OAuthProvider: Send + Sync {
    fn get_inner_provider(&self) -> Box<dyn SimpleOAuthProvider>;
    fn get_credentials(&self) -> OAuthCredentials;
    fn find_linked_user<'a>(
        &self,
        db: &'a mut DbService,
        user_info: &'a UserInfo,
    ) -> BoxFuture<'a, AuthResult<Option<ChatRsUser>>>;
    fn is_user_linked(&self, user: &ChatRsUser) -> bool;
    fn create_update_user<'a>(&self, user_info: &'a UserInfo) -> UpdateChatRsUser<'a>;
    fn create_new_user<'a>(&self, user_info: &'a UserInfo) -> NewChatRsUser<'a>;
}

/// OAuth functions
pub struct OAuthService<'a> {
    config: &'a AppConfig,
    db: &'a DbPool,
    http_client: &'a reqwest::Client,
}

impl<'a> OAuthService<'a> {
    const SESS_STATE_FIELD: &'static str = "oauth_state";
    const SESS_PKCE_FIELD: &'static str = "oauth_verifier";

    pub(super) fn new(
        config: &'a AppConfig,
        db: &'a DbPool,
        http_client: &'a reqwest::Client,
    ) -> Self {
        Self {
            config,
            db,
            http_client,
        }
    }

    pub async fn authorize_url(
        &self,
        provider: OAuthProviderEnum,
        callback_path: &str,
        session: &Session,
    ) -> AuthResult<reqwest::Url> {
        let (client, _) = self.get_client(provider)?;
        let auth = client
            .authorize_url()
            .redirect_url(self.get_redirect_url(callback_path))
            .build()?;

        session.insert(Self::SESS_STATE_FIELD, auth.state).await?;
        session
            .insert(Self::SESS_PKCE_FIELD, auth.pkce_verifier)
            .await?;

        Ok(auth.url)
    }

    pub async fn exchange_code(
        &self,
        provider: OAuthProviderEnum,
        callback_path: &str,
        session: &Session,
        code: &str,
        state: &str,
    ) -> AuthResult<StandardTokenResponse> {
        // Get saved state and code verifier from session
        let initial_state = session
            .remove::<String>(Self::SESS_STATE_FIELD)
            .await?
            .ok_or(AuthError::Unauthorized("missing state in session"))?;
        let pkce_verifier = session
            .remove::<String>(Self::SESS_PKCE_FIELD)
            .await?
            .ok_or(AuthError::Unauthorized("missing PKCE in session"))?;

        // Exchange code for token
        let (client, _) = self.get_client(provider)?;
        let response = client
            .exchange_code()
            .redirect_url(self.get_redirect_url(callback_path))
            .code(code)
            .state(state)
            .initial_state(&initial_state)
            .pkce_verifier(pkce_verifier)
            .build()
            .await?;

        Ok(response)
    }

    pub async fn get_user(
        &self,
        provider: OAuthProviderEnum,
        token: &StandardTokenResponse,
        active_session: Option<UserSession>,
    ) -> AuthResult<ChatRsUser> {
        // Get user info from provider
        let (client, provider) = self.get_client(provider)?;
        let user_info = client.get_user_info(&token.access_token).await?;

        // Check for existing user, or create new user
        let mut db = DbService::from_pool(self.db).await?;
        let user = match provider.find_linked_user(&mut db, &user_info).await? {
            Some(existing_user) => {
                if active_session.is_some_and(|sess| sess.user_id != existing_user.id) {
                    return Err(AuthError::Unauthorized("cannot switch users via OAuth"));
                } else {
                    existing_user
                }
            }
            None => match active_session {
                None => {
                    let new_user = provider.create_new_user(&user_info);
                    db.users().create(new_user).await?
                }
                Some(sess) => match db.users().find_by_id(&sess.user_id).await? {
                    Some(user) if provider.is_user_linked(&user) => {
                        return Err(AuthError::BadRequest("user already linked to provider"));
                    }
                    Some(user) => {
                        // Link logged-in user to new provider
                        let update_user = provider.create_update_user(&user_info);
                        db.users().update(&user.id, update_user).await?;
                        user
                    }
                    None => {
                        return Err(AuthError::UserNotFound);
                    }
                },
            },
        };

        Ok(user)
    }

    fn get_redirect_url(&self, callback_path: &str) -> String {
        format!("{}{}", &self.config.server.base_url, callback_path)
    }

    fn get_client(
        &self,
        provider: OAuthProviderEnum,
    ) -> AuthResult<(
        SimpleOAuthClient<Box<dyn SimpleOAuthProvider>>,
        Box<dyn OAuthProvider>,
    )> {
        let provider: Option<Box<dyn OAuthProvider>> = match provider {
            OAuthProviderEnum::Github => match self.config.auth.github {
                Some(ref c) => Some(Box::new(github::GitHubOAuthProvider::new(c))),
                None => None,
            },
            OAuthProviderEnum::Discord => match self.config.auth.discord {
                Some(ref c) => Some(Box::new(discord::DiscordOAuthProvider::new(c))),
                None => None,
            },
            OAuthProviderEnum::Google => match self.config.auth.google {
                Some(ref c) => Some(Box::new(google::GoogleOAuthProvider::new(c))),
                None => None,
            },
        };

        let provider = provider.ok_or(AuthError::BadRequest("unsupported OAuth provider"))?;
        let client = SimpleOAuthClient::builder()
            .provider(provider.get_inner_provider())
            .credentials(provider.get_credentials())
            .http_client(self.http_client)
            .build()?;

        Ok((client, provider))
    }
}
