use anyhow::Context;
use axum::response::IntoResponse;

use crate::{error::AppResult, extractors::session::UserSession, state::AppState};

pub fn routes() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/login", axum::routing::post(login_handler))
        .route("/user", axum::routing::get(get_user_handler))
        .route("/logout", axum::routing::post(logout_handler))
}

async fn login_handler(session: tower_sessions::Session) -> AppResult<impl IntoResponse> {
    // TODO login handling logic
    let user_id = "user123";
    UserSession::init(&session, user_id).await?;

    Ok(format!("Logged in as {user_id}"))
}

async fn get_user_handler(UserSession { user_id }: UserSession) -> impl IntoResponse {
    format!("Logged in as {user_id}")
}

async fn logout_handler(
    maybe_user: Option<UserSession>,
    session: tower_sessions::Session,
) -> AppResult<impl IntoResponse> {
    match maybe_user {
        Some(_) => {
            session.delete().await.context("error logging out")?;
            Ok("Logged out")
        }
        None => Ok("Already logged out"),
    }
}
