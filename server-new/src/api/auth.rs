use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing,
};
use serde::Deserialize;

use crate::{
    api::RoutePrefix,
    error::AppResult,
    extractors::session::{SessionMeta, UserSession},
    services::auth::oauth::OAuthProviderEnum,
    state::AppState,
};

pub fn routes() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/login/{provider}", routing::get(login_handler))
        .route("/login/{provider}/callback", routing::get(callback_handler))
        .route("/user", routing::get(get_user_handler))
        .route("/logout", routing::post(logout_handler))
}

fn callback_path(route_prefix: &'static str, provider: OAuthProviderEnum) -> String {
    format!("{route_prefix}/login/{}/callback", provider.as_str())
}

async fn login_handler(
    Path(provider): Path<OAuthProviderEnum>,
    Extension(RoutePrefix(prefix)): Extension<RoutePrefix>,
    State(state): State<AppState>,
    session: tower_sessions::Session,
) -> AppResult<impl IntoResponse> {
    let oauth = state.auth_service().oauth();
    let auth_url = oauth
        .authorize_url(provider, &callback_path(prefix, provider), &session)
        .await?;

    Ok(Redirect::to(auth_url.as_str()))
}

#[derive(Debug, Clone, Deserialize)]
struct OAuthCallbackQuery {
    code: String,
    state: String,
}

async fn callback_handler(
    Path(provider): Path<OAuthProviderEnum>,
    Query(query): Query<OAuthCallbackQuery>,
    Extension(RoutePrefix(prefix)): Extension<RoutePrefix>,
    State(state): State<AppState>,
    session: tower_sessions::Session,
    meta: SessionMeta,
    maybe_user: Option<UserSession>,
) -> AppResult<impl IntoResponse> {
    let oauth = state.auth_service().oauth();
    let token = oauth
        .exchange_code(
            provider,
            &callback_path(prefix, provider),
            &session,
            &query.code,
            &query.state,
        )
        .await?;
    let user = oauth.get_user(provider, &token, maybe_user).await?;

    state
        .auth_service()
        .init_session(&session, &meta, &user.id)
        .await?;

    Ok(Redirect::to("/api/auth/user"))
    // Ok(Redirect::to(&state.config.server.base_url))
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
