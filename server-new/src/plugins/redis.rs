use std::time::Duration;

use anyhow::Context;
use axum_plugin::AdHocPlugin;
use fred::prelude::*;

use crate::{config::AppConfig, db::DbPool, state::AppState};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(8);

pub fn plugin() -> AdHocPlugin<AppState> {
    AdHocPlugin::named("Redis").on_init(async |mut state| {
        let app_config = state.get::<AppConfig>().context("no config")?;
        let db_pool = state.get::<DbPool>().context("no database pool")?;
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
            .build_pool(db_pool.status().max_size)?; // same size as database pool

        pool.init().await.context("failed to connect to Redis")?;
        tracing::info!("Connected to Redis");

        state.insert(pool);
        Ok(state)
    })
}
