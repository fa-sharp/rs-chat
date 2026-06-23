use crate::{
    config::AppConfig,
    db::{DbPool, DbService, models::ChatRsUser},
    error::{AppError, AppResult},
    extractors::session::SessionMeta,
};
use tower_sessions::Session;
use uuid::Uuid;

pub mod oauth;
pub mod session_store;

/// The field used to store the user ID in the session
const USER_ID_FIELD: &str = "user_id";
/// The field used to store the user session metadata
const META_FIELD: &str = "meta";

pub struct AuthService<'a> {
    config: &'a AppConfig,
    db: &'a DbPool,
    http_client: &'a reqwest::Client,
}

impl<'a> AuthService<'a> {
    pub fn new(config: &'a AppConfig, http_client: &'a reqwest::Client, db: &'a DbPool) -> Self {
        Self {
            config,
            http_client,
            db,
        }
    }

    /// Initialize a new logged-in session for the given user
    pub async fn init_session(
        &self,
        session: &Session,
        meta: &SessionMeta,
        user_id: &Uuid,
    ) -> AppResult<()> {
        session.insert(USER_ID_FIELD, user_id).await?;
        session.insert(META_FIELD, meta).await?;
        session.set_expiry(Some(tower_sessions::Expiry::OnInactivity(
            tower_sessions::cookie::time::Duration::seconds(self.config.auth.session_length),
        )));

        Ok(())
    }

    /// Extract the current user ID if this is an active user session
    pub async fn extract_user_id(&self, session: Session) -> AppResult<Option<Uuid>> {
        Ok(session.get::<Uuid>(USER_ID_FIELD).await?)
    }

    /// Get the user from the database with the given ID, or return
    /// an internal error if not found
    pub async fn get_user(&self, id: &Uuid) -> AppResult<ChatRsUser> {
        let mut db = DbService::from_pool(&self.db).await?;
        match db.users().find_by_id(id).await? {
            None => Err(AppError::internal(anyhow::anyhow!("user not found"))),
            Some(user) => Ok(user),
        }
    }

    /// Logout the user, deleting the current session
    pub async fn logout_user(&self, session: &Session) -> AppResult<()> {
        session.flush().await?;
        Ok(())
    }

    /// Access OAuth functions
    pub fn oauth(self) -> oauth::OAuthService<'a> {
        oauth::OAuthService {
            config: self.config,
            db: self.db,
            http_client: self.http_client,
        }
    }
}
