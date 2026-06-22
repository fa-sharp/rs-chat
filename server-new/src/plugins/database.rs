use anyhow::Context;
use axum_plugin::AdHocPlugin;
use diesel::Connection;
use diesel_async::{
    AsyncPgConnection,
    pooled_connection::{AsyncDieselConnectionManager, deadpool::Pool},
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};

use crate::{config::AppConfig, db::DbPool, state::AppState};

const MIGRATIONS: EmbeddedMigrations = diesel_migrations::embed_migrations!();

pub fn plugin() -> AdHocPlugin<AppState> {
    AdHocPlugin::named("Database")
        .on_init(async |mut state| {
            let app_config = state.get::<AppConfig>().context("missing config")?;
            let database_url = app_config.database_url.clone();

            tokio::task::spawn_blocking(move || {
                let mut cxn = diesel::PgConnection::establish(&database_url)
                    .context("Failed to connect to database")?;
                tracing::info!("Connected to database at '{database_url}'");
                match cxn.run_pending_migrations(MIGRATIONS) {
                    Ok(run_migrations) => {
                        for migration in run_migrations {
                            tracing::info!("Migration run: '{migration}'");
                        }
                        Ok(())
                    }
                    Err(err) => Err(anyhow::anyhow!(err.to_string()).context("Migration failed")),
                }
            })
            .await??;

            let manager =
                AsyncDieselConnectionManager::<AsyncPgConnection>::new(&app_config.database_url);
            let pool: DbPool = Pool::builder(manager).build()?;

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
