use anyhow::Context;
use diesel::connection::InstrumentationEvent;
use diesel_async::{
    AsyncConnection, AsyncMigrationHarness, AsyncPgConnection,
    pooled_connection::{AsyncDieselConnectionManager, ManagerConfig, deadpool::Pool},
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use futures::TryFutureExt;

use crate::{db::DbPool, plugins::AxumPlugin};

const MIGRATIONS: EmbeddedMigrations = diesel_migrations::embed_migrations!();

pub fn plugin() -> AxumPlugin {
    AxumPlugin::named("Database")
        .on_init(async |mut app| {
            let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(
                &app.config().database.url,
                {
                    let mut config = ManagerConfig::default();
                    config.recycling_method =
                        diesel_async::pooled_connection::RecyclingMethod::Fast;
                    config.custom_setup = Box::new(|url| {
                        Box::pin(AsyncPgConnection::establish(url).map_ok(|mut conn| {
                            conn.set_instrumentation(|ev: InstrumentationEvent<'_>| {
                                if let InstrumentationEvent::FinishQuery { query, error, .. } = ev {
                                    if let Some(err) = error {
                                        tracing::error!(?query, ?err, "Failed to execute query");
                                    } else {
                                        tracing::debug!(?query);
                                    }
                                };
                            });
                            conn
                        }))
                    });
                    config
                },
            );
            let pool: DbPool = Pool::builder(manager).build()?;

            let cxn = pool.get().await.context("failed to connect to database")?;
            tracing::info!("Connected to database");
            match AsyncMigrationHarness::new(cxn).run_pending_migrations(MIGRATIONS) {
                Ok(run_migrations) if run_migrations.is_empty() => {
                    tracing::info!("No migrations to run");
                }
                Ok(run_migrations) => {
                    for migration in run_migrations {
                        tracing::info!("Migration run: '{migration}'");
                    }
                }
                Err(err) => anyhow::bail!(format!("Migrations failed: {err}")),
            };

            app.insert(pool)?;
            Ok(app)
        })
        .on_shutdown(async |app| {
            app.state().db_pool.close();
            tracing::info!("Shut down database pool");
            Ok(())
        })
}
