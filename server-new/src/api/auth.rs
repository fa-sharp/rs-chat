use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

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
    State(state): State<AppState>,
) -> AppResult<impl IntoResponse> {
    if maybe_user.is_some() {
        return Err(AppError::bad_request("already logged in"));
    }

    // TODO login handling logic
    let user_id = uuid::Uuid::parse_str("6976658f-8eef-4a76-ad37-46243f463726").unwrap();
    state
        .auth_service()
        .init_session(&session, &meta, &user_id)
        .await?;

    Ok(format!("Logged in as {user_id}"))
}

async fn get_user_handler(
    UserSession { user_id }: UserSession,
    State(state): State<AppState>,
) -> AppResult<impl IntoResponse> {
    let user = state.auth_service().get_user(&user_id).await?;
    Ok(Json(user))
}

async fn logout_handler(
    session: tower_sessions::Session,
    State(state): State<AppState>,
) -> AppResult<impl IntoResponse> {
    state.auth_service().logout_user(&session).await?;
    Ok(StatusCode::NO_CONTENT)
}
