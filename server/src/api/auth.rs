use rocket::{
    delete, get, post,
    request::{FromRequest, Outcome},
    routes,
    serde::json::Json,
    Route,
};
use rocket_flex_session::Session;
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
    OpenApiFromRequest,
};
use schemars::JsonSchema;

use crate::{
    auth::{
        ChatRsAuthSession, DiscordOAuthConfig, GitHubOAuthConfig, GoogleOAuthConfig, OIDCConfig,
        SSOHeaderMergedConfig,
    },
    db::{
        models::ChatRsUser,
        services::{ApiKeyDbService, ChatDbService, ProviderKeyDbService, UserDbService},
        DbConnection,
    },
    errors::ApiError,
};

/// Auth routes
pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: user, auth_config, logout]
}

/// Undocumented auth routes: account deletion
pub fn get_undocumented_routes() -> Vec<Route> {
    routes![delete_account]
}

/// Get the current user info
#[openapi(tag = "Auth")]
#[get("/user")]
async fn user(user: ChatRsUser) -> Result<Json<ChatRsUser>, ApiError> {
    Ok(Json(user))
}

/// The current auth configuration of the server
#[derive(Debug, JsonSchema, OpenApiFromRequest, serde::Serialize)]
struct AuthConfig {
    /// Whether GitHub login is enabled
    github: bool,
    /// Whether Google login is enabled
    google: bool,
    /// Whether Discord login is enabled
    discord: bool,
    /// OIDC configuration
    oidc: Option<OIDC>,
    /// SSO configuration
    sso: Option<SSO>,
}

#[derive(Debug, JsonSchema, serde::Serialize)]
struct OIDC {
    /// Whether OIDC login is enabled
    enabled: bool,
    /// The name of the OIDC provider
    name: String,
}

#[derive(Debug, JsonSchema, serde::Serialize)]
struct SSO {
    /// Whether SSO header authentication is enabled
    enabled: bool,
    /// The URL to redirect to after logout
    logout_url: Option<String>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthConfig {
    type Error = &'r str;

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let rocket = req.rocket();
        let sso_config = rocket.state::<SSOHeaderMergedConfig>();

        Outcome::Success(AuthConfig {
            github: rocket.state::<GitHubOAuthConfig>().is_some(),
            google: rocket.state::<GoogleOAuthConfig>().is_some(),
            discord: rocket.state::<DiscordOAuthConfig>().is_some(),
            oidc: rocket.state::<OIDCConfig>().map(|config| OIDC {
                enabled: true,
                name: config.oidc_name.clone().unwrap_or("OIDC".to_owned()),
            }),
            sso: sso_config.map(|config| SSO {
                enabled: true,
                logout_url: config.logout_url.clone(),
            }),
        })
    }
}

/// Get the current auth configuration
#[openapi(tag = "Auth")]
#[get("/config")]
async fn auth_config(config: AuthConfig) -> Json<AuthConfig> {
    Json(config)
}

/// Log out
#[openapi(tag = "Auth")]
#[post("/logout")]
async fn logout(mut session: Session<'_, ChatRsAuthSession>) -> Result<String, ApiError> {
    session.delete();

    Ok("Logout successful".to_string())
}

/// Delete account
#[delete("/user/delete-but-only-if-you-are-sure")]
async fn delete_account(user: ChatRsUser, mut db: DbConnection) -> Result<String, ApiError> {
    // Delete Provider keys
    let provider_keys = ProviderKeyDbService::new(&mut db)
        .delete_by_user(&user.id)
        .await?;

    // Delete API keys
    let api_keys = ApiKeyDbService::new(&mut db)
        .delete_by_user(&user.id)
        .await?;

    // Delete chat sessions
    let sessions = ChatDbService::new(&mut db)
        .delete_all_sessions(&user.id)
        .await?;

    let user_id = UserDbService::new(&mut db).delete(&user.id).await?;

    Ok(format!(
        "Deleted user {}: {} sessions, {} provider keys, {} API keys",
        user_id,
        sessions.len(),
        provider_keys.len(),
        api_keys.len()
    ))
}
