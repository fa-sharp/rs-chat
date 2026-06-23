use async_trait::async_trait;
use tower_sessions::{
    SessionStore,
    cookie::time::OffsetDateTime,
    session::{Id, Record},
    session_store::{Error, Result},
};
use uuid::Uuid;

use crate::db::{DbPool, DbService, UtcDateTime};

#[derive(Clone)]
pub struct SessionDbStore {
    db: DbPool,
}

impl SessionDbStore {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    fn get_session_uuid(id: &Id) -> Uuid {
        Uuid::from_bytes(id.0.to_be_bytes())
    }

    fn convert_expiry(time: OffsetDateTime) -> Result<UtcDateTime> {
        UtcDateTime::from_timestamp_secs(time.unix_timestamp())
            .ok_or_else(|| Error::Backend(format!("Invalid expiry: {time}")))
    }

    async fn get_db(&self) -> Result<DbService> {
        DbService::from_pool(&self.db)
            .await
            .map_err(|err| Error::Backend(err.to_string()))
    }
}

impl std::fmt::Debug for SessionDbStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionDbStore").finish()
    }
}

#[async_trait]
impl SessionStore for SessionDbStore {
    /// Creates a new session in the store with the provided session record.
    async fn create(&self, record: &mut Record) -> Result<()> {
        let session_id = Self::get_session_uuid(&record.id);
        let user_id = super::session::user_id_from_record_data(&record.data)?;
        let expires_at = Self::convert_expiry(record.expiry_date)?;

        let mut db = self.get_db().await?;
        db.sessions()
            .create(&session_id, user_id.as_ref(), &record.data, expires_at)
            .await
            .map_err(|err| Error::Backend(err.to_string()))?;

        Ok(())
    }

    /// Saves the provided session record to the store.
    ///
    /// This method is intended for updating the state of an existing session.
    async fn save(&self, record: &Record) -> Result<()> {
        let session_id = Self::get_session_uuid(&record.id);
        let expires_at = Self::convert_expiry(record.expiry_date)?;

        let mut db = self.get_db().await?;
        db.sessions()
            .update(&session_id, &record.data, expires_at)
            .await
            .map_err(|err| Error::Backend(err.to_string()))?;

        Ok(())
    }

    /// Loads an existing session record from the store using the provided ID.
    ///
    /// If a session with the given ID exists, it is returned. If the session
    /// does not exist or has been invalidated (e.g., expired), `None` is
    /// returned.
    async fn load(&self, session_id: &Id) -> Result<Option<Record>> {
        let session_id = Self::get_session_uuid(&session_id);
        let mut db = self.get_db().await?;

        match db.sessions().find_active_by_id(&session_id).await {
            Ok(Some(session)) => Ok(Some(Record {
                id: Id(i128::from_be_bytes(session.id.into_bytes())),
                data: session.data.0,
                expiry_date: OffsetDateTime::from_unix_timestamp(session.expires_at.timestamp())
                    .map_err(|err| Error::Backend(format!("Invalid expiry: {err}")))?,
            })),
            Ok(None) => Ok(None),
            Err(err) => Err(Error::Backend(err.to_string())),
        }
    }

    /// Deletes a session record from the store using the provided ID.
    ///
    /// If the session exists, it is removed from the store.
    async fn delete(&self, session_id: &Id) -> Result<()> {
        let session_id = Self::get_session_uuid(session_id);
        let mut db = self.get_db().await?;

        db.sessions()
            .delete_by_id(&session_id)
            .await
            .map_err(|err| Error::Backend(err.to_string()))?;

        Ok(())
    }
}
