use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    api::{ApiTag, RoutePrefix},
    db::models::ChatRsUser,
    error::AppResult,
    extractors::{CurrentUser, Database, PublicAuthConfig, SessionMeta},
    services::auth::oauth::OAuthProviderEnum,
    state::AppState,
};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_user))
        .routes(routes!(get_config))
        .routes(routes!(oauth_login))
        .routes(routes!(oauth_login_callback))
        .routes(routes!(logout))
}

/// Get current user
#[utoipa::path(
    get, path = "/user",
    responses((status = OK, body = ChatRsUser)),
    tag = ApiTag::Auth.into())
]
async fn get_user(
    CurrentUser { user_id }: CurrentUser,
    Database(mut db): Database,
    State(state): State<AppState>,
) -> AppResult<impl IntoResponse> {
    let user = state.auth_service().get_user(&mut db, &user_id).await?;
    Ok(Json(user))
}

#[utoipa::path(
    get, path = "/config",
    responses((status = OK, body = PublicAuthConfig)),
    tag = ApiTag::Auth.into()
)]
async fn get_config(auth_config: PublicAuthConfig) -> impl IntoResponse {
    Json(auth_config)
}

fn oauth_callback_path(route_prefix: &'static str, provider: OAuthProviderEnum) -> String {
    format!("{route_prefix}/login/{provider}/callback")
}

/// OAuth login redirect
#[utoipa::path(
    get, path = "/login/{provider}",
    params(("provider" = OAuthProviderEnum, Path)),
    responses((status = OK)),
    tag = ApiTag::Auth.into(),
)]
async fn oauth_login(
    Path(provider): Path<OAuthProviderEnum>,
    Extension(RoutePrefix(prefix)): Extension<RoutePrefix>,
    State(state): State<AppState>,
    session: tower_sessions::Session,
) -> AppResult<impl IntoResponse> {
    let oauth = state.auth_service().oauth();
    let auth_url = oauth
        .authorize_url(provider, &oauth_callback_path(prefix, provider), &session)
        .await?;

    Ok(Redirect::to(auth_url.as_str()))
}

#[derive(Debug, Clone, Deserialize, IntoParams, ToSchema)]
struct OAuthCallbackQuery {
    code: String,
    state: String,
}

/// OAuth login callback
#[utoipa::path(
    get, path = "/login/{provider}/callback",
    params(
        ("query" = inline(OAuthCallbackQuery), Query),
        ("provider" = OAuthProviderEnum, Path, description = "the OAuth provider")
    ),
    responses((status = OK)),
    tag = ApiTag::Auth.into(),
)]
async fn oauth_login_callback(
    Path(provider): Path<OAuthProviderEnum>,
    Query(query): Query<OAuthCallbackQuery>,
    Extension(RoutePrefix(prefix)): Extension<RoutePrefix>,
    Database(mut db): Database,
    State(state): State<AppState>,
    session: tower_sessions::Session,
    meta: SessionMeta,
    maybe_user: Option<CurrentUser>,
) -> AppResult<impl IntoResponse> {
    let oauth = state.auth_service().oauth();
    let token = oauth
        .exchange_code(
            provider,
            &oauth_callback_path(prefix, provider),
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

/// Logout
#[utoipa::path(
    method(get, post), path = "/logout",
    tag = ApiTag::Auth.into(),
    responses((status = NO_CONTENT)),
)]
async fn logout(
    session: tower_sessions::Session,
    State(state): State<AppState>,
) -> AppResult<impl IntoResponse> {
    state.auth_service().session().logout(&session).await?;
    Ok(StatusCode::NO_CONTENT)
}
