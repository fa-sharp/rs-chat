use std::time::Duration;

use anyhow::Context;
use fred::prelude::*;

use crate::plugins::AxumPlugin;

pub fn plugin() -> AxumPlugin {
    AxumPlugin::named("Redis")
        .on_init(async |mut app| {
            let config = Config::from_url(&app.config().redis.url).context("parse Redis URL")?;
            let timeout = Duration::from_secs(app.config().redis.timeout);

            let pool = Builder::from_config(config)
                .with_connection_config(|c| {
                    c.connection_timeout = timeout;
                    c.internal_command_timeout = timeout;
                    c.tcp.nodelay = Some(true);
                })
                .with_performance_config(|c| {
                    c.default_command_timeout = timeout;
                })
                .build_pool(app.config().redis.pool_size)?;

            pool.init().await.context("failed to connect to Redis")?;
            tracing::info!("Connected to Redis");
            app.insert(pool)?;

            Ok(app)
        })
        .on_shutdown(async |app| {
            if let Err(e) = app.state().redis.quit().await {
                tracing::warn!("Error shutting down Redis pool: {e}");
            } else {
                tracing::info!("Shut down Redis pool")
            }

            Ok(())
        })
}
