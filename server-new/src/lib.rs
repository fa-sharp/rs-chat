use axum_plugin::{App, InitializedApp};

use crate::state::AppState;

mod api;
mod config;
mod db;
mod error;
mod extractors;
mod plugins;
mod services;
mod state;

pub async fn create_app() -> anyhow::Result<InitializedApp<AppState>> {
    let http_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let app = App::new()
        .store(http_client) // Add shared http client
        .register(config::plugin()) // Extract configuration and add to state
        .register(plugins::database::plugin()) // Initialize database
        .register(plugins::redis::plugin()) // Initialize Redis
        .register(api::plugin()) // Add API routes
        .register(plugins::session::plugin()) // Setup sessions
        .register(plugins::logging::plugin()) // Request logging
        .register(plugins::security::plugin()) // Body limit, security headers, etc.
        .init()
        .await?;

    Ok(app)
}
