use std::collections::HashMap;

use tower_sessions::{
    Expiry, Session,
    cookie::time::Duration,
    session_store::{Error as StoreError, Result as StoreResult},
};
use uuid::Uuid;

use crate::{
    db::{DbPool, DbService},
    services::auth::{
        AuthResult,
        types::{SessionMeta, UserSession},
    },
};

/// The field used to store the user ID in the session.
const USER_ID_FIELD: &str = "user_id";
/// The field used to store the user session metadata.
const META_FIELD: &str = "meta";

/// Authentication-specific operations on a tower session.
pub struct AuthSessionService {
    session_length: i64,
}

impl AuthSessionService {
    pub fn new(session_length: i64) -> Self {
        Self { session_length }
    }

    /// Initialize a new logged-in session for the given user.
    pub async fn login(
        &self,
        session: &Session,
        meta: &SessionMeta,
        user_id: &Uuid,
    ) -> AuthResult<()> {
        session.cycle_id().await?; // ensures that the user id is saved to the database
        session.insert(USER_ID_FIELD, user_id).await?;
        session.insert(META_FIELD, meta).await?;
        session.set_expiry(Some(Expiry::OnInactivity(Duration::seconds(
            self.session_length,
        ))));

        Ok(())
    }

    /// Extract the current user session if this is an active user session.
    pub async fn user_session(&self, session: &Session) -> AuthResult<Option<UserSession>> {
        let user_id = session.get::<Uuid>(USER_ID_FIELD).await?;
        Ok(user_id.map(UserSession::new))
    }

    /// Logout the user, deleting the current session.
    pub async fn logout(&self, session: &Session) -> AuthResult<()> {
        Ok(session.flush().await?)
    }

    // Cleanup expired sessions
    #[tracing::instrument(skip(db_pool), level = "debug")]
    pub async fn session_cleanup(db_pool: &DbPool) -> AuthResult<usize> {
        let mut db = DbService::from_pool(&db_pool).await?;
        Ok(db.sessions().delete_expired().await?)
    }
}

pub(super) fn user_id_from_record_data(
    data: &HashMap<String, serde_json::Value>,
) -> StoreResult<Option<Uuid>> {
    data.get(USER_ID_FIELD)
        .map(|val| serde_json::from_value::<Uuid>(val.clone()))
        .transpose()
        .map_err(|_| StoreError::Encode("invalid user id field".to_owned()))
}
