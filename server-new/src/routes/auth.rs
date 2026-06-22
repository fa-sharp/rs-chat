use anyhow::Context;
use axum::{extract::State, response::IntoResponse};

use crate::{
    error::{AppError, AppResult},
    extractors::session::{SessionMeta, UserSession},
    state::AppState,
};

pub fn routes() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/login", axum::routing::post(login_handler))
        .route("/user", axum::routing::get(get_user_handler))
        .route("/logout", axum::routing::post(logout_handler))
}

async fn login_handler(
    maybe_user: Option<UserSession>,
    session: tower_sessions::Session,
    meta: SessionMeta,
) -> AppResult<impl IntoResponse> {
    if maybe_user.is_some() {
        return Err(AppError::bad_request("already logged in"));
    }

    // TODO login handling logic
    let user_id = uuid::Uuid::new_v4();
    UserSession::init(&session, &meta, &user_id).await?;

    Ok(format!("Logged in as {user_id}"))
}

async fn get_user_handler(
    UserSession { user_id }: UserSession,
    State(state): State<AppState>,
) -> AppResult<impl IntoResponse> {
    state.user_service().get_user(&user_id).await?;

    Ok(format!("Logged in as {user_id}"))
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
