use futures::future::BoxFuture;
use serde::Deserialize;

use crate::{
    db::models::{ChatRsUser, NewChatRsUser, UpdateChatRsUser},
    error::AppResult,
    services::auth::oauth::OAuthProvider,
};

#[derive(Clone, Debug, Deserialize)]
pub struct DiscordOAuthConfig {
    client_id: u64,
    client_secret: String,
}

pub struct DiscordOAuthProvider {
    config: DiscordOAuthConfig,
}

impl DiscordOAuthProvider {
    pub fn new(config: &DiscordOAuthConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

impl OAuthProvider for DiscordOAuthProvider {
    fn get_authorize_url(&self) -> &str {
        "https://discord.com/oauth2/authorize"
    }

    fn get_token_url(&self) -> &str {
        "https://discord.com/api/oauth2/token"
    }

    fn get_scopes(&self) -> Vec<&str> {
        vec!["identify"]
    }

    fn get_user_info_url(&self) -> &str {
        "https://discord.com/api/v9/users/@me"
    }

    fn get_client_id(&self) -> String {
        self.config.client_id.to_string()
    }

    fn get_client_secret(&self) -> String {
        self.config.client_secret.clone()
    }

    fn extract_user_data(&self, user_info: serde_json::Value) -> anyhow::Result<super::UserData> {
        let user_info: DiscordUserInfo = serde_json::from_value(user_info)?;
        let avatar_url = user_info.avatar.as_ref().map(|avatar| {
            format!(
                "https://cdn.discordapp.com/avatars/{}/{}.png",
                user_info.id, avatar
            )
        });

        Ok(super::UserData {
            id: user_info.id,
            name: user_info.global_name.unwrap_or_else(|| user_info.username),
            avatar_url,
        })
    }

    fn find_linked_user<'a>(
        &self,
        db: &'a mut crate::db::DbService,
        user_data: &'a super::UserData,
    ) -> BoxFuture<'a, AppResult<Option<ChatRsUser>>> {
        Box::pin(async move {
            let user = db.users().find_by_discord_id(&user_data.id).await?;
            Ok(user)
        })
    }

    fn is_user_linked(&self, user: &ChatRsUser) -> bool {
        user.discord_id.is_some()
    }

    fn create_update_user<'a>(&self, user_data: &'a super::UserData) -> UpdateChatRsUser<'a> {
        UpdateChatRsUser {
            discord_id: Some(&user_data.id),
            ..Default::default()
        }
    }

    fn create_new_user<'a>(&self, user_data: &'a super::UserData) -> NewChatRsUser<'a> {
        NewChatRsUser {
            discord_id: Some(&user_data.id),
            name: &user_data.name,
            avatar_url: user_data.avatar_url.as_deref(),
            ..Default::default()
        }
    }
}

/// User info returned from Discord API
#[derive(Debug, Deserialize)]
pub struct DiscordUserInfo {
    id: String,
    username: String,
    global_name: Option<String>,
    avatar: Option<String>,
}
