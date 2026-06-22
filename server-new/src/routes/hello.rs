use crate::{error::AppResult, state::AppState};

pub fn routes() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", axum::routing::get(hello_handler))
        .route("/", axum::routing::post(post_handler))
}

async fn hello_handler() -> AppResult<String> {
    Ok("Hello, World!".to_string())
}

async fn post_handler() -> AppResult<String> {
    Ok("Post handler!".to_string())
}
