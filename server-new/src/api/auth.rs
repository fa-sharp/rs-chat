use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use serde::Deserialize;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    api::RoutePrefix,
    db::models::ChatRsUser,
    error::AppResult,
    extractors::{
        database::Database,
        session::{SessionMeta, UserSession},
    },
    services::auth::oauth::OAuthProviderEnum,
    state::AppState,
};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(login_handler, logout_handler))
        .routes(routes!(login_callback_handler))
        .routes(routes!(get_user_handler))
}

fn callback_path(route_prefix: &'static str, provider: OAuthProviderEnum) -> String {
    format!("{route_prefix}/login/{}/callback", provider.as_str())
}

#[utoipa::path(get, path = "/login/{provider}", params(("provider" = OAuthProviderEnum, Path)), responses((status = OK)))]
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

#[utoipa::path(get, path = "/login/{provider}/callback", params(("provider" = OAuthProviderEnum, Path)), responses((status = OK)))]
async fn login_callback_handler(
    Path(provider): Path<OAuthProviderEnum>,
    Query(query): Query<OAuthCallbackQuery>,
    Extension(RoutePrefix(prefix)): Extension<RoutePrefix>,
    Database(mut db): Database,
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
    let user = oauth
        .get_user(&mut db, provider, &token, maybe_user)
        .await?;
    state
        .auth_service()
        .session()
        .login(&session, &meta, &user.id)
        .await?;

    Ok(Redirect::to(&state.config.server.base_url))
}

#[utoipa::path(get, path = "/user", responses((status = OK, body = ChatRsUser)))]
async fn get_user_handler(
    UserSession { user_id }: UserSession,
    Database(mut db): Database,
    State(state): State<AppState>,
) -> AppResult<impl IntoResponse> {
    let user = state.auth_service().get_user(&mut db, &user_id).await?;
    Ok(Json(user))
}

#[utoipa::path(get, post, path = "/logout", responses((status = NO_CONTENT)))]
async fn logout_handler(
    session: tower_sessions::Session,
    State(state): State<AppState>,
) -> AppResult<impl IntoResponse> {
    state.auth_service().session().logout(&session).await?;
    Ok(StatusCode::NO_CONTENT)
}
