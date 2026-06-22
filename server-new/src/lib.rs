use axum_plugin::{App, InitializedApp};

use crate::state::AppState;

mod config;
mod error;
mod extractors;
mod plugins;
mod routes;
mod state;

pub async fn create_app() -> anyhow::Result<InitializedApp<AppState>> {
    let app = App::new()
        .register(config::plugin()) // Extract configuration and add to state
        .register(routes::plugin()) // Add API routes
        .register(plugins::session::plugin()) // Setup sessions
        .register(plugins::logging::plugin()) // Request logging
        .register(plugins::security::plugin()) // Body limit, security headers, etc.
        .init()
        .await?;

    Ok(app)
}
