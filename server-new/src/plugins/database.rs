use anyhow::Context;
use axum_plugin::AdHocPlugin;
use diesel_async::{
    AsyncMigrationHarness, AsyncPgConnection,
    pooled_connection::{AsyncDieselConnectionManager, ManagerConfig, deadpool::Pool},
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};

use crate::{config::AppConfig, db::DbPool, state::AppState};

const MIGRATIONS: EmbeddedMigrations = diesel_migrations::embed_migrations!();

pub fn plugin() -> AdHocPlugin<AppState> {
    AdHocPlugin::named("Database")
        .on_init(async |mut state| {
            let app_config = state.get::<AppConfig>().context("missing config")?;

            let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(
                &app_config.database.url,
                {
                    let mut config = ManagerConfig::default();
                    config.recycling_method =
                        diesel_async::pooled_connection::RecyclingMethod::Fast;
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

            state.insert(pool);
            Ok(state)
        })
        .on_shutdown(|state: &AppState| {
            let pool = state.db_pool.clone();
            async move {
                pool.close();
                tracing::info!("Shut down database pool");
                Ok(())
            }
        })
}
