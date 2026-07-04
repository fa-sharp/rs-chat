use axum::Extension;
use axum_plugin::AdHocPlugin;
use strum::{AsRefStr, IntoStaticStr};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

use crate::{services::auth::oauth::OAuthProviderEnum, state::AppState};

pub mod api_key;
pub mod auth;
pub mod chat;
pub mod health;

#[derive(AsRefStr, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
enum ApiTag {
    ApiKey,
    Auth,
    Chat,
}

#[derive(OpenApi)]
#[openapi(
    servers((url = "/api")),
    components(
        schemas(OAuthProviderEnum)
    ),
    tags(
        (name = ApiTag::ApiKey.as_ref(), description = "Manage API keys"),
        (name = ApiTag::Auth.as_ref(), description = "Authentication"),
        (name = ApiTag::Chat.as_ref(), description = "Chats and sessions")
    )
)]
struct ApiDoc;

/// Adds all API routes with OpenAPI docs to the server under `/api`
pub fn plugin() -> AdHocPlugin<AppState> {
    AdHocPlugin::named("API routes").on_setup(|router, _state| {
        let (api_routes, openapi) = OpenApiRouter::with_openapi(ApiDoc::openapi())
            .nest("/api_key", api_key::routes())
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
struct RoutePrefix(&'static str);
