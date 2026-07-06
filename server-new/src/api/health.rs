use aide::axum::ApiRouter;
use axum_typed_routing::{TypedApiRouter, api_route};

use crate::state::AppState;

pub fn routes() -> ApiRouter<AppState> {
    ApiRouter::new().typed_api_route(health)
}

#[api_route(GET "/" with AppState { summary: "Health route" })]
async fn health() -> &'static str {
    "OK"
}
