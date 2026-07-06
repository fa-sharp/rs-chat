use aide::axum::{ApiRouter, routing::get_with};
use aide_docs_macro::docs;
use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Redirect,
};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    api::{ApiTag, RoutePrefix},
    db::models::ChatRsUser,
    error::AppResult,
    extractors::{AppSession, CurrentUser, Database, PublicAuthConfig},
    services::auth::oauth::OAuthProviderEnum,
    state::AppState,
};

pub fn routes() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/user", get_with(get_user, get_user_docs))
        .api_route("/config", get_with(get_config, get_config_docs))
        .api_route("/login", get_with(oauth_login, oauth_login_docs))
        .api_route(
            "/login/callback",
            get_with(oauth_callback, oauth_callback_docs),
        )
        .api_route(
            "/logout",
            get_with(logout, logout_docs).post_with(logout, logout_docs),
        )
        .with_path_items(|op| op.tag(ApiTag::Auth.into()))
}

#[docs("Get user", "Get the current user")]
async fn get_user(
    CurrentUser { user_id }: CurrentUser,
    Database(mut db): Database,
    State(state): State<AppState>,
) -> AppResult<Json<ChatRsUser>> {
    let user = state.auth_service().get_user(&mut db, &user_id).await?;
    Ok(Json(user))
}

#[docs("Get auth config", "Get the current auth configuration of the server")]
async fn get_config(auth_config: PublicAuthConfig) -> Json<PublicAuthConfig> {
    Json(auth_config)
}

fn oauth_callback_path(route_prefix: &'static str, provider: &OAuthProviderEnum) -> String {
    format!("{route_prefix}/login/{provider}/callback")
}

#[docs("OAuth login", "OAuth login redirect")]
async fn oauth_login(
    Path(provider): Path<OAuthProviderEnum>,
    Extension(RoutePrefix(prefix)): Extension<RoutePrefix>,
    State(state): State<AppState>,
    AppSession { session, .. }: AppSession,
) -> AppResult<Redirect> {
    let oauth = state.auth_service().oauth();
    let auth_url = oauth
        .authorize_url(&provider, &oauth_callback_path(prefix, &provider), &session)
        .await?;

    Ok(Redirect::to(auth_url.as_str()))
}

#[derive(Debug, Deserialize, JsonSchema)]
struct OAuthCallbackQuery {
    code: String,
    state: String,
}

#[docs("OAuth login callback sdf")]
async fn oauth_callback(
    Path(provider): Path<OAuthProviderEnum>,
    Query(query): Query<OAuthCallbackQuery>,
    maybe_user: Option<CurrentUser>,
    Extension(RoutePrefix(prefix)): Extension<RoutePrefix>,
    AppSession { session, meta }: AppSession,
    Database(mut db): Database,
    State(app_state): State<AppState>,
) -> AppResult<Redirect> {
    let oauth = app_state.auth_service().oauth();
    let token = oauth
        .exchange_code(
            &provider,
            &oauth_callback_path(prefix, &provider),
            &session,
            &query.code,
            &query.state,
        )
        .await?;
    let user = oauth
        .get_user(&mut db, &provider, &token, maybe_user)
        .await?;
    app_state
        .auth_service()
        .session()
        .login(&session, &meta, &user.id)
        .await?;

    Ok(Redirect::to(&app_state.config.server.base_url))
}

#[docs("Logout")]
async fn logout(
    AppSession { session, .. }: AppSession,
    State(state): State<AppState>,
) -> AppResult<StatusCode> {
    state.auth_service().session().logout(&session).await?;
    Ok(StatusCode::NO_CONTENT)
}
