use std::fmt::Debug;

use crate::{
    db::{DbPool, DbService, models::ChatRsUser},
    error::{AppError, AppResult},
    extractors::session::SessionMeta,
};
use tower_sessions::Session;
use uuid::Uuid;

mod session_store;
pub use session_store::SessionDbStore;

/// The field used to store the user ID in the session
const USER_ID_FIELD: &str = "user_id";
/// The field used to store the user session metadata
const META_FIELD: &str = "meta";

pub struct AuthService<'a> {
    db: &'a DbPool,
}

impl<'a> Debug for AuthService<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthService").finish()
    }
}

impl<'a> AuthService<'a> {
    pub fn new(db: &'a DbPool) -> Self {
        Self { db }
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
}
