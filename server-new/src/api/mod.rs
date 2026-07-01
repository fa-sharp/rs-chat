use axum::Extension;
use axum_plugin::AdHocPlugin;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

use crate::{services::auth::oauth::OAuthProviderEnum, state::AppState};

pub mod auth;
pub mod chat;
pub mod health;

#[derive(OpenApi)]
#[openapi(
    servers((url = "/api")),
    components(
        schemas(OAuthProviderEnum)
    ),
    tags(
        (name = "chat", description = "Chat routes")
    )
)]
struct ApiDoc;

/// Adds all API routes with OpenAPI docs to the server under `/api`
pub fn plugin() -> AdHocPlugin<AppState> {
    AdHocPlugin::named("API routes").on_setup(|router, _state| {
        let (api_routes, openapi) = OpenApiRouter::with_openapi(ApiDoc::openapi())
            .nest(
                "/auth",
                auth::routes().layer(Extension(RoutePrefix("/api/auth"))),
            )
            .nest("/chat", chat::routes())
            .nest("/health", health::routes())
            .split_for_parts();

        Ok(router.nest("/api", api_routes.merge(Scalar::with_url("/docs", openapi))))
    })
}

#[derive(Clone)]
struct RoutePrefix(pub &'static str);
