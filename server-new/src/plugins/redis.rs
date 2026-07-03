use std::time::Duration;

use anyhow::Context;
use axum_plugin::AdHocPlugin;
use fred::prelude::*;

use crate::{config::AppConfig, state::AppState};

pub fn plugin() -> AdHocPlugin<AppState> {
    AdHocPlugin::named("Redis")
        .on_init(async |mut state| {
            let app_config = state.get::<AppConfig>().context("no config")?;
            let config = Config::from_url(&app_config.redis.url).context("invalid Redis URL")?;
            let timeout = Duration::from_secs(app_config.redis.timeout);

            let pool = Builder::from_config(config)
                .with_connection_config(|c| {
                    c.connection_timeout = timeout;
                    c.internal_command_timeout = timeout;
                    c.tcp.nodelay = Some(true);
                })
                .with_performance_config(|c| {
                    c.default_command_timeout = timeout;
                })
                .build_pool(app_config.redis.pool_size)?;

            pool.init().await.context("failed to connect to Redis")?;
            tracing::info!("Connected to Redis");

            state.insert(pool);
            Ok(state)
        })
        .on_shutdown(|state: &AppState| {
            let pool = state.redis.clone();
            async move {
                if let Err(e) = pool.quit().await {
                    tracing::warn!("Error shutting down Redis pool: {e}");
                } else {
                    tracing::info!("Shut down Redis pool")
                }
                Ok(())
            }
        })
}
