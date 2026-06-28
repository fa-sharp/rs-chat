use std::time::Duration;

use anyhow::Context;
use axum_plugin::AdHocPlugin;
use fred::prelude::*;

use crate::{config::AppConfig, state::AppState};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(8);
const POOL_SIZE: usize = 4;

pub fn plugin() -> AdHocPlugin<AppState> {
    AdHocPlugin::named("Redis")
        .on_init(async |mut state| {
            let app_config = state.get::<AppConfig>().context("no config")?;
            let config = Config::from_url(&app_config.redis.url).context("invalid Redis URL")?;
            let pool = Builder::from_config(config)
                .with_connection_config(|c| {
                    c.connection_timeout = DEFAULT_TIMEOUT;
                    c.internal_command_timeout = DEFAULT_TIMEOUT;
                    c.tcp.nodelay = Some(true);
                })
                .with_performance_config(|c| {
                    c.default_command_timeout = DEFAULT_TIMEOUT;
                })
                .build_pool(POOL_SIZE)?;

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
