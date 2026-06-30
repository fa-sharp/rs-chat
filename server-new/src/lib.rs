use axum_plugin::{App, InitializedApp};

use crate::state::AppState;

mod api;
mod config;
mod db;
mod error;
mod extractors;
mod llm;
mod plugins;
mod services;
mod state;

pub async fn create_app() -> anyhow::Result<InitializedApp<AppState>> {
    let app = App::new()
        .register(config::plugin()) // Extract configuration and add to state
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
