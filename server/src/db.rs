pub mod models;
pub mod schema;
pub mod services;

use std::ops::{Deref, DerefMut};

use diesel_async::{
    pooled_connection::{
        deadpool::{Object, Pool},
        AsyncDieselConnectionManager,
    },
    AsyncPgConnection,
};
use diesel_async_migrations::{embed_migrations, EmbeddedMigrations};
use rocket::{
    fairing::AdHoc,
    http::Status,
    request::{FromRequest, Outcome},
    Request,
};
use rocket_okapi::OpenApiFromRequest;

use crate::config::get_app_config;

/// Database connection, available as a request guard. When used as a request parameter,
/// it will retrieve a connection from the managed Postgres pool.
#[derive(OpenApiFromRequest)]
pub struct DbConnection(pub Object<AsyncPgConnection>);
impl Deref for DbConnection {
    type Target = Object<AsyncPgConnection>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for DbConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Retrieve a connection from the managed Postgres pool. Responds with an
/// internal server error if a connection couldn't be retrieved.
#[rocket::async_trait]
impl<'r> FromRequest<'r> for DbConnection {
    type Error = &'static str;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let Some(pool) = req.rocket().state::<DbPool>() else {
            return Outcome::Error((Status::InternalServerError, "Database not initialized"));
        };
        match pool.get().await {
            Ok(conn) => Outcome::Success(DbConnection(conn)),
            Err(e) => {
                rocket::error!("Couldn't get database connection: {}", e);
                Outcome::Error((Status::InternalServerError, "Couldn't get connection"))
            }
        }
    }
}

/** The database pool stored in Rocket's managed state */
pub type DbPool = Pool<AsyncPgConnection>;

/// Fairing that sets up and initializes the Postgres database
pub fn setup_db() -> AdHoc {
    AdHoc::on_ignite("Database", |rocket| async {
        rocket
            .attach(AdHoc::on_ignite(
                "Initialize database connection",
                |rocket| async {
                    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(
                        &get_app_config(&rocket).database_url,
                    );
                    let pool: DbPool = Pool::builder(config)
                        .build()
                        .expect("Failed to parse database URL");
                    let mut conn = pool.get().await.expect("Failed to connect to database");

                    static MIGRATIONS: EmbeddedMigrations = embed_migrations!();
                    MIGRATIONS
                        .pending_migrations(&mut conn)
                        .await
                        .expect("Failed to get pending migrations")
                        .iter()
                        .for_each(|migration| {
                            rocket::info!("Running migration: {}", migration.name);
                        });
                    MIGRATIONS
                        .run_pending_migrations(&mut conn)
                        .await
                        .expect("Database migrations failed");
                    rocket::info!("Migrations completed successfully");

                    rocket.manage(pool)
                },
            ))
            .attach(AdHoc::on_shutdown(
                "Shutdown database connection",
                |rocket| {
                    Box::pin(async {
                        if let Some(pool) = rocket.state::<DbPool>() {
                            rocket::info!("Shutting down database connection");
                            pool.close();
                        }
                    })
                },
            ))
    })
}
