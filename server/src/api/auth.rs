use rocket::{get, http::CookieJar, post, response::Redirect, routes, serde::json::Json, Route};
use rocket_flex_session::Session;
use rocket_oauth2::{OAuth2, TokenResponse};
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
};

use crate::{
    auth::ChatRsAuthSession,
    db::{
        models::{ChatRsUser, NewChatRsUser},
        services::user::UserDbService,
        DbConnection,
    },
    errors::ApiError,
};

pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: user, logout]
}

/// OAuth routes for GitHub authentication - not included in OpenAPI spec.
pub fn get_oauth_routes() -> Vec<Route> {
    routes![login, login_callback]
}

/// User information to be retrieved from the GitHub API.
#[derive(serde::Deserialize)]
pub struct GitHubUserInfo {
    id: u64,
    #[serde(default)]
    name: String,
}

#[get("/login/github")]
async fn login(
    oauth2: OAuth2<GitHubUserInfo>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, ApiError> {
    oauth2
        .get_redirect(cookies, &["user:read"])
        .map_err(|e| ApiError::Authentication(format!("Failed to get redirect: {}", e)))
}

#[get("/login/github/callback")]
async fn login_callback(
    mut db: DbConnection,
    token: TokenResponse<GitHubUserInfo>,
    mut session: Session<'_, ChatRsAuthSession>,
) -> Result<Redirect, ApiError> {
    let user_info_res = reqwest::Client::builder()
        .build()
        .map_err(|e| ApiError::Authentication(format!("Failed to build reqwest client: {}", e)))?
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token.access_token()))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "fa-sharp/chat-rs")
        .send()
        .await
        .map_err(|e| ApiError::Authentication(format!("Failed to get GitHub user: {}", e)))?;
    let user_info: GitHubUserInfo = user_info_res
        .json()
        .await
        .map_err(|e| ApiError::Authentication(format!("Failed to deserialize response: {}", e)))?;

    let mut db_service = UserDbService::new(&mut db);
    match db_service.find_by_github_id(user_info.id).await? {
        Some(existing_user) => {
            session.set(ChatRsAuthSession {
                user_id: existing_user.id,
            });
        }
        None => {
            let new_user = db_service
                .create(NewChatRsUser {
                    github_id: &user_info.id.to_string(),
                    name: &user_info.name,
                })
                .await?;
            session.set(ChatRsAuthSession {
                user_id: new_user.id,
            });
        }
    }

    Ok(Redirect::to("/"))
}

/// Get the current user info
#[openapi(tag = "User")]
#[get("/user")]
async fn user(user: ChatRsUser) -> Result<Json<ChatRsUser>, ApiError> {
    Ok(Json(user))
}

/// Log out
#[openapi(tag = "User")]
#[post("/logout")]
async fn logout(mut session: Session<'_, ChatRsAuthSession>) -> Result<String, ApiError> {
    session.delete();

    Ok("Logout successful".to_string())
}
