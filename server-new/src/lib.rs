use axum_plugin::{App, InitializedApp};

use crate::{config::AppConfig, plugins::AxumPlugin, state::AppState};

mod api;
mod config;
mod db;
mod error;
mod extractors;
mod llm;
mod plugins;
mod services;
mod state;

pub async fn create_app() -> anyhow::Result<InitializedApp<AppState, AppConfig>> {
    let app = App::from_figment(config::figment())?
        .register(AxumPlugin::named("Config").on_init(async |app| {
            let config = app.config();
            tracing::info!(
                log_level = config.server.log_level,
                base_url = config.server.base_url,
                host = %config.server.host,
                port = config.server.port,
                "Config loaded!"
            );
            Ok(app)
        }))
        .register(plugins::clients::plugin()) // Initialize HTTP clients
        .register(plugins::database::plugin()) // Initialize database
        .register(plugins::redis::plugin()) // Initialize Redis
        .register(api::plugin()) // Add API routes
        .register(plugins::auth::plugin()) // Setup auth & sessions
        .register(plugins::logging::plugin()) // Request logging
        .register(plugins::security::plugin()) // Body limit, security headers, etc.
        .init()
        .await?;

    Ok(app)
}
