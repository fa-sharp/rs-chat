use rocket::{get, http::CookieJar, response::Redirect, routes, Route, State};
use rocket_flex_session::Session;
use rocket_oauth2::{OAuth2, StaticProvider, TokenResponse};
use serde::Deserialize;

use crate::{
    db::{
        models::{ChatRsUser, NewChatRsUser, UpdateChatRsUser},
        services::UserDbService,
        DbConnection,
    },
    errors::ApiError,
};

use super::{generic_login, generic_login_callback, ChatRsAuthSession, OAuthProvider, UserData};

/// Discord OAuth provider
pub struct DiscordProvider {
    config: DiscordOAuthConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DiscordOAuthConfig {
    discord_client_id: u64,
    discord_client_secret: String,
}

#[derive(Debug, Deserialize)]
pub struct DiscordUserInfo {
    id: String,
    username: String,
    global_name: Option<String>,
    avatar: Option<String>,
}

impl OAuthProvider for DiscordProvider {
    type Config = DiscordOAuthConfig;
    type UserInfo = DiscordUserInfo;

    const PROVIDER_NAME: &'static str = "Discord";

    fn new(config: &Self::Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    fn get_static_provider(&self) -> StaticProvider {
        StaticProvider::Discord
    }

    fn get_scopes(&self) -> Vec<&str> {
        vec!["identify"]
    }

    fn get_routes() -> Vec<Route> {
        routes![discord_login, discord_login_callback]
    }

    fn get_user_info_url(&self) -> &str {
        "https://discord.com/api/v9/users/@me"
    }

    fn get_client_id(&self) -> String {
        self.config.discord_client_id.to_string()
    }

    fn get_client_secret(&self) -> String {
        self.config.discord_client_secret.clone()
    }

    fn create_request_headers() -> Vec<(&'static str, &'static str)> {
        vec![]
    }

    fn extract_user_data(user_info: Self::UserInfo) -> UserData {
        let avatar_url = user_info.avatar.as_ref().map(|avatar| {
            format!(
                "https://cdn.discordapp.com/avatars/{}/{}.png",
                user_info.id, avatar
            )
        });

        UserData {
            id: user_info.id,
            name: user_info.global_name.unwrap_or_else(|| user_info.username),
            avatar_url,
        }
    }

    async fn find_linked_user(
        db: &mut UserDbService<'_>,
        user_data: &UserData,
    ) -> Result<Option<ChatRsUser>, ApiError> {
        Ok(db.find_by_discord_id(&user_data.id).await?)
    }

    fn is_user_linked(user: &ChatRsUser) -> bool {
        user.discord_id.is_some()
    }

    fn create_update_user(user_data: &UserData) -> UpdateChatRsUser {
        UpdateChatRsUser {
            discord_id: Some(&user_data.id),
            ..Default::default()
        }
    }

    fn create_new_user(user_data: &UserData) -> NewChatRsUser {
        NewChatRsUser {
            discord_id: Some(&user_data.id),
            name: &user_data.name,
            avatar_url: user_data.avatar_url.as_deref(),
            ..Default::default()
        }
    }
}

#[get("/login/discord")]
async fn discord_login(
    oauth2: OAuth2<DiscordUserInfo>,
    cookies: &CookieJar<'_>,
    config: &State<DiscordOAuthConfig>,
) -> Result<Redirect, ApiError> {
    generic_login::<DiscordProvider>(oauth2, cookies, config, Some(&[("prompt", "none")]))
}

#[get("/login/discord/callback")]
async fn discord_login_callback(
    db: DbConnection,
    token: TokenResponse<DiscordUserInfo>,
    config: &State<DiscordOAuthConfig>,
    session: Session<'_, ChatRsAuthSession>,
) -> Result<Redirect, ApiError> {
    generic_login_callback::<DiscordProvider>(db, token, config, session).await
}
