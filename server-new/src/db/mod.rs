use anyhow::Context;

use crate::error::AppResult;

pub mod models;
mod repositories;
mod schema;

/// Type of the database pool
pub type DbPool = diesel_async::pooled_connection::deadpool::Pool<diesel_async::AsyncPgConnection>;
/// Type of the database connection retrieved from the pool
pub type DbConnection =
    diesel_async::pooled_connection::deadpool::Object<diesel_async::AsyncPgConnection>;

/// Wrapper around a database connection that gives access to the repositories,
/// e.g. `UserRepository`, `ChatRepository`, etc.
pub struct DbService {
    cxn: DbConnection,
}

impl DbService {
    pub fn new(cxn: DbConnection) -> Self {
        Self { cxn }
    }

    pub async fn from_pool(pool: &DbPool) -> AppResult<Self> {
        let cxn = pool.get().await.context("error retrieving DB connection")?;
        Ok(Self::new(cxn))
    }

    pub fn users(&mut self) -> repositories::UserRepository<'_> {
        repositories::UserRepository::new(&mut self.cxn)
    }
}
