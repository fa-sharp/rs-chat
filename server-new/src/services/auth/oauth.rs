use std::collections::HashMap;

use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use simple_oauth::{
    SimpleOAuthClient, SimpleOAuthError, SimpleOAuthProvider,
    types::{OAuthCredentials, StandardTokenResponse, UserInfo},
};
use strum::Display;
use tower_sessions::Session;
use utoipa::ToSchema;

use crate::{
    config::AppConfig,
    db::{
        DbService,
        models::{ChatRsUser, NewChatRsUser, UpdateChatRsUser},
    },
    extractors::CurrentUser,
    services::auth::{AuthError, AuthResult},
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
pub type OAuthProviderMap = HashMap<OAuthProviderEnum, (OAuthClient, Box<dyn OAuthProvider>)>;
/// Type of the OAuth client stored in state
pub type OAuthClient = SimpleOAuthClient<Box<dyn SimpleOAuthProvider>>;

/// Supported OAuth provider
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum OAuthProviderEnum {
    Github,
    Discord,
    Google,
    Oidc,
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
    provider_map: &'a OAuthProviderMap,
}

impl<'a> OAuthService<'a> {
    const SESS_STATE_FIELD: &'static str = "oauth_state";
    const SESS_PKCE_FIELD: &'static str = "oauth_verifier";

    pub(super) fn new(config: &'a AppConfig, provider_map: &'a OAuthProviderMap) -> Self {
        Self {
            config,
            provider_map,
        }
    }

    fn oauth_provider(
        &self,
        provider: OAuthProviderEnum,
    ) -> AuthResult<(&OAuthClient, &Box<dyn OAuthProvider>)> {
        let (client, provider) = self
            .provider_map
            .get(&provider)
            .ok_or_else(|| AuthError::BadRequest("unsupported OAuth provider"))?;
        Ok((client, provider))
    }

    fn get_redirect_url(&self, callback_path: &str) -> String {
        format!("{}{}", &self.config.server.base_url, callback_path)
    }

    pub async fn authorize_url(
        &self,
        provider: OAuthProviderEnum,
        callback_path: &str,
        session: &Session,
    ) -> AuthResult<reqwest::Url> {
        let (oauth_client, _) = self.oauth_provider(provider)?;
        let auth = oauth_client
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
        let (oauth_client, _) = self.oauth_provider(provider)?;
        let response = oauth_client
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
        db: &mut DbService,
        provider: OAuthProviderEnum,
        token: &StandardTokenResponse,
        active_session: Option<CurrentUser>,
    ) -> AuthResult<ChatRsUser> {
        // Get user info from provider
        let (oauth_client, oauth_provider) = self.oauth_provider(provider)?;
        let user_info = oauth_client.get_user_info(&token.access_token).await?;

        // Check for existing user, or create new user
        let user = match oauth_provider.find_linked_user(db, &user_info).await? {
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

    pub fn build_provider_map(
        config: &crate::config::AppConfig,
        http_client: &reqwest::Client,
    ) -> Result<OAuthProviderMap, SimpleOAuthError> {
        use {
            discord::DiscordProvider, github::GitHubProvider, google::GoogleProvider,
            oidc::OidcProvider,
        };

        let mut map: OAuthProviderMap = HashMap::new();
        if let Some(ref c) = config.auth.github {
            let provider = GitHubProvider::new(c);
            let client = Self::build_oauth_client(http_client, &provider)?;
            map.insert(OAuthProviderEnum::Github, (client, Box::new(provider)));
        }
        if let Some(ref c) = config.auth.discord {
            let provider = DiscordProvider::new(c);
            let client = Self::build_oauth_client(http_client, &provider)?;
            map.insert(OAuthProviderEnum::Discord, (client, Box::new(provider)));
        }
        if let Some(ref c) = config.auth.google {
            let provider = GoogleProvider::new(c);
            let client = Self::build_oauth_client(http_client, &provider)?;
            map.insert(OAuthProviderEnum::Google, (client, Box::new(provider)));
        }
        if let Some(ref c) = config.auth.oidc {
            let provider = OidcProvider::new(c);
            let client = Self::build_oauth_client(http_client, &provider)?;
            map.insert(OAuthProviderEnum::Oidc, (client, Box::new(provider)));
        }

        Ok(map)
    }

    fn build_oauth_client(
        http_client: &reqwest::Client,
        provider: &impl OAuthProvider,
    ) -> Result<SimpleOAuthClient<Box<dyn SimpleOAuthProvider>>, SimpleOAuthError> {
        let oauth_client = SimpleOAuthClient::builder()
            .provider(provider.get_inner_provider())
            .credentials(provider.get_credentials())
            .redirect_url("http://example.com/should-be-overridden")
            .http_client(http_client)
            .build()?;
        Ok(oauth_client)
    }
}
