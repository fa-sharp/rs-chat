use crate::{
    config::AppConfig,
    db::{DbPool, DbService, models::ChatRsUser},
};
use uuid::Uuid;

mod error;
pub mod oauth;
pub mod session;
pub mod session_store;
mod types;

pub use error::{AuthError, AuthResult};
pub use types::*;

pub struct AuthService<'a> {
    db: &'a DbPool,
    config: &'a AppConfig,
    http_client: &'a reqwest::Client,
    oauth_providers: &'a oauth::OAuthProviderMap,
}

impl<'a> AuthService<'a> {
    pub fn new(
        db: &'a DbPool,
        config: &'a AppConfig,
        http_client: &'a reqwest::Client,
        oauth_providers: &'a oauth::OAuthProviderMap,
    ) -> Self {
        Self {
            db,
            config,
            http_client,
            oauth_providers,
        }
    }

    /// Get the user from the database with the given ID, or return
    /// an internal error if not found
    pub async fn get_user(&self, id: &Uuid) -> AuthResult<ChatRsUser> {
        let mut db = DbService::from_pool(&self.db).await?;
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
        oauth::OAuthService::new(self.config, self.db, self.http_client, self.oauth_providers)
    }
}
