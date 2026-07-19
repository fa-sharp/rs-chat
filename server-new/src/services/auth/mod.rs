use crate::{
    config::AppConfig,
    db::{DbService, models::ChatRsUser},
    services::auth::encryption::Encryptor,
};
use uuid::Uuid;

pub mod api_key;
pub mod encryption;
mod error;
pub mod oauth;
pub mod proxy;
pub mod session;
pub mod session_store;

use error::{AuthError, AuthResult};

pub struct AuthService<'r> {
    config: &'r AppConfig,
    encryptor: &'r Encryptor,
    oauth_providers: &'r oauth::OAuthProviderMap,
}

impl<'r> AuthService<'r> {
    pub fn new(
        config: &'r AppConfig,
        encryptor: &'r Encryptor,
        oauth_providers: &'r oauth::OAuthProviderMap,
    ) -> Self {
        Self {
            config,
            encryptor,
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
    pub fn oauth(self) -> oauth::OAuthService<'r> {
        oauth::OAuthService::new(self.config, self.oauth_providers)
    }

    /// Access proxy auth functions
    pub fn proxy(&self) -> proxy::ProxyService<'r> {
        proxy::ProxyService::new(&self.config.auth.proxy)
    }

    /// Access API key functions
    pub fn api_keys(&self) -> api_key::ApiKeyService<'r> {
        api_key::ApiKeyService::new(self.encryptor)
    }
}
