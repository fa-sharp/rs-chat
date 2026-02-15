pub mod models;
pub mod schema;
pub mod services;

use std::ops::{Deref, DerefMut};

use diesel_async::{
    async_connection_wrapper::AsyncConnectionWrapper,
    pooled_connection::{
        deadpool::{Object, Pool},
        AsyncDieselConnectionManager,
    },
    AsyncPgConnection,
};

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use rocket::{
    fairing::AdHoc,
    http::Status,
    request::{FromRequest, Outcome},
    Request,
};
use rocket_okapi::OpenApiFromRequest;

use crate::config::get_app_config;

/** The PostgreSQL connection pool, stored in Rocket's managed state */
pub type DbPool = Pool<AsyncPgConnection>;

/// Database connection, available as a request guard. When used as a request parameter,
/// it will retrieve a connection from the managed Postgres pool.
#[derive(OpenApiFromRequest)]
pub struct DbConnection(pub Object<AsyncPgConnection>);
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

#[rocket::async_trait]
impl<'r> FromRequest<'r> for DbConnection {
    type Error = &'static str;

    /// Retrieve a connection from the managed Postgres pool. Responds with an
    /// internal server error if a connection couldn't be retrieved.
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let pool = req.rocket().state::<DbPool>().expect("should be attached");
        match pool.get().await {
            Ok(conn) => Outcome::Success(DbConnection(conn)),
            Err(e) => {
                rocket::error!("Couldn't get database connection: {e}");
                Outcome::Error((Status::InternalServerError, "Couldn't get connection"))
            }
        }
    }
}

/// Fairing that sets up and initializes the Postgres database
pub fn setup_db() -> AdHoc {
    AdHoc::on_ignite("Database", |rocket| async {
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(
            &get_app_config(&rocket).database_url,
        );
        let pool: DbPool = Pool::builder(config)
            .build()
            .expect("Failed to parse database URL");

        const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
        let cxn = pool.get().await.expect("Failed to connect to database");
        tokio::task::spawn_blocking(move || {
            AsyncConnectionWrapper::<Object<AsyncPgConnection>>::from(cxn)
                .run_pending_migrations(MIGRATIONS)
                .expect("Database migrations failed");
        })
        .await
        .expect("Database migration task failed");

        rocket::info!("Migrations completed successfully");

        let shutdown = AdHoc::on_shutdown("Shutdown database", |rocket| {
            Box::pin(async {
                if let Some(pool) = rocket.state::<DbPool>() {
                    rocket::info!("Shutting down database connection");
                    pool.close();
                }
            })
        });

        rocket.manage(pool).attach(shutdown)
    })
}
