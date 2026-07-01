use crate::{
    config::AppConfig,
    db::{DbService, models::ChatRsUser},
};
use uuid::Uuid;

pub mod encryption;
mod error;
pub mod oauth;
pub mod session;
pub mod session_store;

use error::{AuthError, AuthResult};

pub struct AuthService<'a> {
    config: &'a AppConfig,
    http_client: &'a reqwest::Client,
    oauth_providers: &'a oauth::OAuthProviderMap,
}

impl<'a> AuthService<'a> {
    pub fn new(
        config: &'a AppConfig,
        http_client: &'a reqwest::Client,
        oauth_providers: &'a oauth::OAuthProviderMap,
    ) -> Self {
        Self {
            config,
            http_client,
            oauth_providers,
        }
    }

    /// Get the user from the database with the given ID, or return
    /// an internal error if not found
    pub async fn get_user(&self, db: &mut DbService, id: &Uuid) -> AuthResult<ChatRsUser> {
        match db.users().find_by_id(id).await? {
            None => Err(AuthError::UserNotFound),
            Some(user) => Ok(user),
        }
    }

    /// Access session functions.
    pub fn session(&self) -> session::AuthSessionService {
        session::AuthSessionService::new(self.config.auth.session_length)
    }

    /// Access OAuth functions
    pub fn oauth(self) -> oauth::OAuthService<'a> {
        oauth::OAuthService::new(self.config, self.http_client, self.oauth_providers)
    }
}
