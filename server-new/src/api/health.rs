use aide::axum::{ApiRouter, routing::get_with};
use aide_docs_macro::handler_docs;

use crate::state::AppState;

pub fn routes() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/", get_with(health, health_docs))
}

#[handler_docs("Health route")]
async fn health() -> &'static str {
    "OK"
}
