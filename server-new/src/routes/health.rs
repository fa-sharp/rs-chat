use crate::state::AppState;

pub fn routes() -> axum::Router<AppState> {
    axum::Router::new().route("/", axum::routing::get(health_handler))
}

async fn health_handler() -> &'static str {
    "OK"
}
