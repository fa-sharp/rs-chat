use std::sync::Arc;

use aide::{axum::ApiRouter, openapi::OpenApi, swagger::Swagger};
use axum::{Extension, http::header, response::IntoResponse, routing::get};
use axum_plugin::AdHocPlugin;
use strum::{AsRefStr, Display, EnumIter, EnumMessage, IntoEnumIterator, IntoStaticStr};

use crate::state::AppState;

pub mod api_key;
pub mod auth;
pub mod chat;
pub mod health;

#[derive(Display, AsRefStr, IntoStaticStr, EnumMessage, EnumIter)]
enum ApiTag {
    #[strum(message = "Manage API keys")]
    ApiKey,
    #[strum(message = "Authentication")]
    Auth,
    #[strum(message = "Chats and sessions")]
    Chat,
}

/// Adds all API routes with OpenAPI docs to the server under `/api`
pub fn plugin() -> AdHocPlugin<AppState> {
    AdHocPlugin::named("API routes").on_setup(|router, _state| {
        let mut openapi = OpenApi::default();
        for tag in ApiTag::iter() {
            openapi.tags.push(aide::openapi::Tag {
                name: tag.to_string(),
                description: tag.get_message().map(String::from),
                ..Default::default()
            })
        }

        let api_routes = ApiRouter::new()
            .nest("/api_key", api_key::routes())
            .nest(
                "/auth",
                auth::routes().layer(Extension(RoutePrefix("/api/auth"))),
            )
            .nest("/chat", chat::routes())
            .nest("/health", health::routes())
            .finish_api(&mut openapi);

        let api_routes_with_docs = api_routes
            .route(
                "/docs/openapi.json",
                get(openapi_route).layer(Extension(Arc::new(openapi))),
            )
            .route(
                "/docs",
                get(Swagger::new("/api/docs/openapi.json")
                    .with_title("RsChat API")
                    .axum_handler()),
            );

        Ok(router.nest("/api", api_routes_with_docs))
    })
}

async fn openapi_route(Extension(openapi): Extension<Arc<OpenApi>>) -> impl IntoResponse {
    axum::Json(openapi)
}

/// Extension to pass the route prefix to child routes
#[derive(Clone)]
struct RoutePrefix(&'static str);
