use utoipa_axum::{router::OpenApiRouter, routes};

use crate::state::AppState;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(health_handler))
}

#[utoipa::path(get, path = "", responses((status = OK, body = &str)))]
async fn health_handler() -> &'static str {
    "OK"
}
