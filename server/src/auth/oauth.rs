use rocket::{fairing::AdHoc, get, http::CookieJar, response::Redirect, routes};
use rocket_flex_session::Session;
use rocket_oauth2::{HyperRustlsAdapter, OAuth2, OAuthConfig, StaticProvider, TokenResponse};
use serde::Deserialize;

use crate::{
    auth::ChatRsAuthSession,
    config::{get_app_config, get_config_provider},
    db::{
        models::{NewChatRsUser, UpdateChatRsUser},
        services::user::UserDbService,
        DbConnection,
    },
    errors::ApiError,
};

// OAuth config / environment variables
#[derive(Debug, Deserialize)]
pub struct GitHubOAuthConfig {
    pub github_client_id: String,
    pub github_client_secret: String,
}

#[derive(Debug, Deserialize)]
pub struct GoogleOAuthConfig {
    pub google_client_id: String,
    pub google_client_secret: String,
}

#[derive(Debug, Deserialize)]
pub struct DiscordOAuthConfig {
    pub discord_client_id: u64,
    pub discord_client_secret: String,
}

// OAuth user info responses
#[derive(Debug, Deserialize)]
struct GitHubUserInfo {
    id: u64,
    login: String,
    name: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GoogleUserInfo {
    sub: String,
    name: String,
    picture: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DiscordUserInfo {
    id: String,
    username: String,
    global_name: Option<String>,
    avatar: Option<String>,
}

/// Fairing that sets up OAuth login and routes
pub fn setup_oauth(base_path: &str) -> AdHoc {
    let base_path = base_path.to_owned();
    let config_provider = get_config_provider();

    AdHoc::on_ignite("OAuth", |mut rocket| async move {
        if let Ok(github_config) = config_provider.extract::<GitHubOAuthConfig>() {
            rocket::info!("OAuth: GitHub login enabled!");
            let oauth_config = OAuthConfig::new(
                StaticProvider::GitHub,
                github_config.github_client_id.to_owned(),
                github_config.github_client_secret.to_owned(),
                Some(format!(
                    "{}{}/login/github/callback",
                    get_app_config(&rocket).server_address,
                    base_path,
                )),
            );
            rocket = rocket
                .manage(github_config)
                .mount(&base_path, routes![github_login, github_login_callback])
                .attach(rocket_oauth2::OAuth2::<GitHubUserInfo>::custom(
                    HyperRustlsAdapter::default(),
                    oauth_config,
                ));
        }
        if let Ok(google_config) = config_provider.extract::<GoogleOAuthConfig>() {
            rocket::info!("OAuth: Google login enabled!");
            let oauth_config = OAuthConfig::new(
                StaticProvider::Google,
                google_config.google_client_id.to_owned(),
                google_config.google_client_secret.to_owned(),
                Some(format!(
                    "{}{}/login/google/callback",
                    get_app_config(&rocket).server_address,
                    base_path,
                )),
            );
            rocket = rocket
                .manage(google_config)
                .mount(&base_path, routes![google_login, google_login_callback])
                .attach(rocket_oauth2::OAuth2::<GoogleUserInfo>::custom(
                    HyperRustlsAdapter::default(),
                    oauth_config,
                ));
        }
        if let Ok(discord_config) = config_provider.extract::<DiscordOAuthConfig>() {
            rocket::info!("OAuth: Discord login enabled!");
            let oauth_config = OAuthConfig::new(
                StaticProvider::Discord,
                discord_config.discord_client_id.to_string(),
                discord_config.discord_client_secret.to_owned(),
                Some(format!(
                    "{}{}/login/discord/callback",
                    get_app_config(&rocket).server_address,
                    base_path,
                )),
            );
            rocket = rocket
                .manage(discord_config)
                .mount(&base_path, routes![discord_login, discord_login_callback])
                .attach(rocket_oauth2::OAuth2::<DiscordUserInfo>::custom(
                    HyperRustlsAdapter::default(),
                    oauth_config,
                ));
        }

        rocket
    })
}

#[get("/login/github")]
async fn github_login(
    oauth2: OAuth2<GitHubUserInfo>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, ApiError> {
    oauth2
        .get_redirect(cookies, &["user:read"])
        .map_err(|e| ApiError::Authentication(format!("Failed to get redirect: {}", e)))
}

#[get("/login/github/callback")]
async fn github_login_callback(
    mut db: DbConnection,
    token: TokenResponse<GitHubUserInfo>,
    mut session: Session<'_, ChatRsAuthSession>,
) -> Result<Redirect, ApiError> {
    let user_info: GitHubUserInfo = reqwest::Client::builder()
        .build()
        .map_err(|e| ApiError::Authentication(format!("Failed to build reqwest client: {}", e)))?
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token.access_token()))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "fa-sharp/rs-chat")
        .send()
        .await
        .map_err(|e| ApiError::Authentication(format!("Failed to get GitHub user: {}", e)))?
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
        None => match session.get() {
            Some(session_data) => {
                db_service
                    .update(
                        &session_data.user_id,
                        UpdateChatRsUser {
                            github_id: Some(&user_info.id.to_string()),
                            ..Default::default()
                        },
                    )
                    .await?;
            }
            None => {
                let new_user = db_service
                    .create(NewChatRsUser {
                        github_id: Some(&user_info.id.to_string()),
                        name: user_info
                            .name
                            .as_deref()
                            .unwrap_or_else(|| user_info.login.as_str()),
                        ..Default::default()
                    })
                    .await?;
                session.set(ChatRsAuthSession {
                    user_id: new_user.id,
                });
            }
        },
    }

    Ok(Redirect::to("/"))
}

#[get("/login/google")]
async fn google_login(
    oauth2: OAuth2<GoogleUserInfo>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, ApiError> {
    oauth2
        .get_redirect(cookies, &["openid", "profile"])
        .map_err(|e| ApiError::Authentication(format!("Failed to get redirect: {}", e)))
}

#[get("/login/google/callback")]
async fn google_login_callback(
    mut db: DbConnection,
    token: TokenResponse<GoogleUserInfo>,
    mut session: Session<'_, ChatRsAuthSession>,
) -> Result<Redirect, ApiError> {
    let user_info: GoogleUserInfo = reqwest::Client::builder()
        .build()
        .map_err(|e| ApiError::Authentication(format!("Failed to build reqwest client: {}", e)))?
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .header("Authorization", format!("Bearer {}", token.access_token()))
        .send()
        .await
        .map_err(|e| ApiError::Authentication(format!("Failed to get Google user: {}", e)))?
        .json()
        .await
        .map_err(|e| ApiError::Authentication(format!("Failed to deserialize response: {}", e)))?;

    let mut db_service = UserDbService::new(&mut db);
    match db_service.find_by_google_id(&user_info.sub).await? {
        Some(existing_user) => {
            session.set(ChatRsAuthSession {
                user_id: existing_user.id,
            });
        }
        None => match session.get() {
            Some(session_data) => {
                db_service
                    .update(
                        &session_data.user_id,
                        UpdateChatRsUser {
                            google_id: Some(&user_info.sub),
                            ..Default::default()
                        },
                    )
                    .await?;
            }
            None => {
                let new_user = db_service
                    .create(NewChatRsUser {
                        google_id: Some(&user_info.sub),
                        name: &user_info.name,
                        ..Default::default()
                    })
                    .await?;
                session.set(ChatRsAuthSession {
                    user_id: new_user.id,
                });
            }
        },
    }

    Ok(Redirect::to("/"))
}

#[get("/login/discord")]
async fn discord_login(
    oauth2: OAuth2<DiscordUserInfo>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, ApiError> {
    oauth2
        .get_redirect(cookies, &["identify"])
        .map_err(|e| ApiError::Authentication(format!("Failed to get redirect: {}", e)))
}

#[get("/login/discord/callback")]
async fn discord_login_callback(
    mut db: DbConnection,
    token: TokenResponse<DiscordUserInfo>,
    mut session: Session<'_, ChatRsAuthSession>,
) -> Result<Redirect, ApiError> {
    let user_info: DiscordUserInfo = reqwest::Client::builder()
        .build()
        .map_err(|e| ApiError::Authentication(format!("Failed to build reqwest client: {}", e)))?
        .get("https://discord.com/api/v9/users/@me")
        .header("Authorization", format!("Bearer {}", token.access_token()))
        .send()
        .await
        .map_err(|e| ApiError::Authentication(format!("Failed to get Discord user: {}", e)))?
        .json()
        .await
        .map_err(|e| ApiError::Authentication(format!("Failed to deserialize response: {}", e)))?;

    let mut db_service = UserDbService::new(&mut db);
    match db_service.find_by_discord_id(&user_info.id).await? {
        Some(existing_user) => {
            session.set(ChatRsAuthSession {
                user_id: existing_user.id,
            });
        }
        None => match session.get() {
            Some(session_data) => {
                db_service
                    .update(
                        &session_data.user_id,
                        UpdateChatRsUser {
                            discord_id: Some(&user_info.id),
                            ..Default::default()
                        },
                    )
                    .await?;
            }
            None => {
                let new_user = db_service
                    .create(NewChatRsUser {
                        discord_id: Some(&user_info.id),
                        name: &user_info.global_name.unwrap_or(user_info.username),
                        ..Default::default()
                    })
                    .await?;
                session.set(ChatRsAuthSession {
                    user_id: new_user.id,
                });
            }
        },
    }

    Ok(Redirect::to("/"))
}
