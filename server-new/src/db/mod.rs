pub mod models;
mod repositories;
mod schema;

/// Type of the database pool
pub type DbPool = diesel_async::pooled_connection::deadpool::Pool<diesel_async::AsyncPgConnection>;
/// Error when attempting to retrieve a connection from the pool
pub type DbPoolError = diesel_async::pooled_connection::deadpool::PoolError;
/// Type of the database connection retrieved from the pool
pub type DbConnection =
    diesel_async::pooled_connection::deadpool::Object<diesel_async::AsyncPgConnection>;

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
        Ok(Self::new(cxn))
    }

    pub fn users(&mut self) -> repositories::UserRepository<'_> {
        repositories::UserRepository::new(&mut self.cxn)
    }

    pub fn sessions(&mut self) -> repositories::SessionRepository<'_> {
        repositories::SessionRepository::new(&mut self.cxn)
    }
}
