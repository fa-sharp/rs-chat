use std::collections::HashMap;

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
mod oidc;

pub use discord::DiscordOAuthConfig;
pub use github::GitHubOAuthConfig;
pub use google::GoogleOAuthConfig;
pub use oidc::OidcConfig;

/// Map of configured OAuth providers stored in state
pub type OAuthProviderMap = HashMap<OAuthProviderEnum, Box<dyn OAuthProvider>>;

/// Supported OAuth providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OAuthProviderEnum {
    Github,
    Discord,
    Google,
    Oidc,
}
impl OAuthProviderEnum {
    pub fn as_str(&self) -> &str {
        match self {
            OAuthProviderEnum::Github => "github",
            OAuthProviderEnum::Discord => "discord",
            OAuthProviderEnum::Google => "google",
            OAuthProviderEnum::Oidc => "oidc",
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
    provider_map: &'a OAuthProviderMap,
}

impl<'a> OAuthService<'a> {
    const SESS_STATE_FIELD: &'static str = "oauth_state";
    const SESS_PKCE_FIELD: &'static str = "oauth_verifier";

    pub(super) fn new(
        config: &'a AppConfig,
        db: &'a DbPool,
        http_client: &'a reqwest::Client,
        provider_map: &'a OAuthProviderMap,
    ) -> Self {
        Self {
            config,
            db,
            http_client,
            provider_map,
        }
    }

    pub async fn authorize_url(
        &self,
        provider: OAuthProviderEnum,
        callback_path: &str,
        session: &Session,
    ) -> AuthResult<reqwest::Url> {
        let oauth_provider = self.oauth_provider(provider)?;
        let auth = self
            .oauth_client(oauth_provider)?
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
        let oauth_provider = self.oauth_provider(provider)?;
        let response = self
            .oauth_client(oauth_provider)?
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
        let oauth_provider = self.oauth_provider(provider)?;
        let user_info = self
            .oauth_client(oauth_provider)?
            .get_user_info(&token.access_token)
            .await?;

        // Check for existing user, or create new user
        let mut db = DbService::from_pool(self.db).await?;
        let user = match oauth_provider.find_linked_user(&mut db, &user_info).await? {
            Some(existing_user) => {
                if active_session.is_some_and(|sess| sess.user_id != existing_user.id) {
                    return Err(AuthError::Unauthorized("cannot switch users via OAuth"));
                } else {
                    existing_user
                }
            }
            None => match active_session {
                None => {
                    let new_user = oauth_provider.create_new_user(&user_info);
                    db.users().create(new_user).await?
                }
                Some(sess) => match db.users().find_by_id(&sess.user_id).await? {
                    Some(user) if oauth_provider.is_user_linked(&user) => {
                        return Err(AuthError::BadRequest("user already linked to provider"));
                    }
                    Some(user) => {
                        // Link logged-in user to new provider
                        let update_user = oauth_provider.create_update_user(&user_info);
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

    fn oauth_provider(&self, provider: OAuthProviderEnum) -> AuthResult<&dyn OAuthProvider> {
        let provider = self
            .provider_map
            .get(&provider)
            .ok_or_else(|| AuthError::BadRequest("unsupported OAuth provider"))?;
        Ok(provider.as_ref())
    }

    fn oauth_client(
        &self,
        provider: &dyn OAuthProvider,
    ) -> Result<SimpleOAuthClient<Box<dyn SimpleOAuthProvider>>, AuthError> {
        Ok(simple_oauth::SimpleOAuthClient::builder()
            .provider(provider.get_inner_provider())
            .credentials(provider.get_credentials())
            .http_client(self.http_client)
            .build()?)
    }

    pub fn build_provider_map(config: &crate::config::AuthConfig) -> OAuthProviderMap {
        use {
            discord::DiscordProvider, github::GitHubProvider, google::GoogleProvider,
            oidc::OidcProvider,
        };
        let mut map: OAuthProviderMap = HashMap::new();
        if let Some(ref c) = config.github {
            map.insert(OAuthProviderEnum::Github, Box::new(GitHubProvider::new(c)));
        }
        if let Some(ref c) = config.discord {
            map.insert(
                OAuthProviderEnum::Discord,
                Box::new(DiscordProvider::new(c)),
            );
        }
        if let Some(ref c) = config.google {
            map.insert(OAuthProviderEnum::Google, Box::new(GoogleProvider::new(c)));
        }
        if let Some(ref c) = config.oidc {
            map.insert(OAuthProviderEnum::Oidc, Box::new(OidcProvider::new(c)));
        }

        map
    }
}
