use axum_plugin::AdHocPlugin;

use crate::state::AppState;

pub mod auth;
pub mod health;
pub mod hello;

/// Adds all API routes to the server under `/api`
pub fn plugin() -> AdHocPlugin<AppState> {
    AdHocPlugin::named("API routes").on_setup(|router, _state| {
        let api_routes = axum::Router::new()
            .nest("/auth", auth::routes())
            .nest("/hello", hello::routes())
            .nest("/health", health::routes());

        Ok(router.nest("/api", api_routes))
    })
}
