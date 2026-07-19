//! Database operations

use std::ops::{Deref, DerefMut};

use diesel_async::{
    AsyncPgConnection,
    pooled_connection::deadpool::{Object, Pool, PoolError},
};

pub mod models;
pub mod queries;
pub mod repositories;
mod schema;

/// Type of the database pool
pub type DbPool = Pool<AsyncPgConnection>;
/// Error when attempting to retrieve a connection from the pool
pub type DbPoolError = PoolError;

/// The database connection retrieved from the pool. For pipelining multiple
/// queries in Diesel, a shared reference can be used with `&mut conn.as_ref()`.
pub struct DbConnection(Object<AsyncPgConnection>);
impl Deref for DbConnection {
    type Target = AsyncPgConnection;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for DbConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl AsRef<AsyncPgConnection> for DbConnection {
    fn as_ref(&self) -> &AsyncPgConnection {
        &**self
    }
}

/// Date/time format used in all database tables
pub type UtcDateTime = chrono::DateTime<chrono::Utc>;

/// Wrapper around a database connection that gives access to the repositories,
/// e.g. `UserRepository`, `ChatRepository`, etc.
pub struct DbService {
    cxn: DbConnection,
}

impl DbService {
    pub fn new(cxn: DbConnection) -> Self {
        Self { cxn }
    }
    pub async fn from_pool(pool: &DbPool) -> Result<Self, DbPoolError> {
        let cxn = pool.get().await?;
        Ok(Self::new(DbConnection(cxn)))
    }

    pub fn api_keys(&mut self) -> repositories::ApiKeyRepository<'_> {
        repositories::ApiKeyRepository::new(&mut self.cxn)
    }
    pub fn auth_sessions(&mut self) -> repositories::SessionRepository<'_> {
        repositories::SessionRepository::new(&mut self.cxn)
    }
    pub fn chats(&mut self) -> repositories::ChatRepository<'_> {
        repositories::ChatRepository::new(&mut self.cxn)
    }
    pub fn files(&mut self) -> repositories::FileRepository<'_> {
        repositories::FileRepository::new(&mut self.cxn)
    }
    pub fn logs(&mut self) -> repositories::LogRepository<'_> {
        repositories::LogRepository::new(&mut self.cxn)
    }
    pub fn providers(&mut self) -> repositories::ProviderRepository<'_> {
        repositories::ProviderRepository::new(&mut self.cxn)
    }
    pub fn secrets(&mut self) -> repositories::SecretRepository<'_> {
        repositories::SecretRepository::new(&mut self.cxn)
    }
    pub fn users(&mut self) -> repositories::UserRepository<'_> {
        repositories::UserRepository::new(&mut self.cxn)
    }
}
