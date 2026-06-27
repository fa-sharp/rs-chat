use serde::Deserialize;

use crate::{SimpleOAuthProvider, types::UserInfo};

pub struct Discord {
    client_id: u64,
    client_secret: String,
}

impl Discord {
    pub fn new(client_id: u64, client_secret: impl Into<String>) -> Self {
        Self {
            client_id,
            client_secret: client_secret.into(),
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

impl SimpleOAuthProvider for Discord {
    fn get_authorize_url(&self) -> String {
        "https://discord.com/oauth2/authorize".to_owned()
    }

    fn get_token_url(&self) -> String {
        "https://discord.com/api/oauth2/token".to_owned()
    }

    fn get_scopes(&self) -> Vec<String> {
        vec!["identify".to_owned()]
    }

    fn get_user_info_url(&self) -> String {
        "https://discord.com/api/v9/users/@me".to_owned()
    }

    fn get_client_id(&self) -> String {
        self.client_id.to_string()
    }

    fn get_client_secret(&self) -> String {
        self.client_secret.clone()
    }

    fn extract_user_info(&self, val: serde_json::Value) -> Result<UserInfo, serde_json::Error> {
        let user_info: DiscordUserInfo = serde_json::from_value(val)?;
        let avatar_url = user_info.avatar.as_ref().map(|avatar| {
            format!(
                "https://cdn.discordapp.com/avatars/{}/{}.png",
                user_info.id, avatar
            )
        });

        Ok(UserInfo {
            id: user_info.id,
            name: user_info.global_name.unwrap_or_else(|| user_info.username),
            avatar_url,
        })
    }
}
