use axum::{Extension, Router};
use axum_plugin::AdHocPlugin;

use crate::state::AppState;

pub mod auth;
pub mod health;
pub mod hello;

/// Adds all API routes to the server under `/api`
pub fn plugin() -> AdHocPlugin<AppState> {
    AdHocPlugin::named("API routes").on_setup(|router, _state| {
        let api_routes = Router::new()
            .nest(
                "/auth",
                auth::routes().layer(Extension(RoutePrefix("/api/auth"))),
            )
            .nest("/hello", hello::routes())
            .nest("/health", health::routes());

        Ok(router.nest("/api", api_routes))
    })
}

#[derive(Clone)]
struct RoutePrefix(pub &'static str);
