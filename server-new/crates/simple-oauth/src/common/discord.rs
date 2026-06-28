use serde::Deserialize;

use crate::{SimpleOAuthProvider, types::UserInfo};

#[derive(Debug)]
pub struct Discord;

/// User info returned from Discord API
#[derive(Debug, Deserialize)]
struct DiscordUserInfo {
    id: String,
    username: String,
    global_name: Option<String>,
    email: Option<String>,
    verified: Option<bool>,
    avatar: Option<String>,
}

impl SimpleOAuthProvider for Discord {
    fn authorize_url(&self) -> &str {
        "https://discord.com/oauth2/authorize"
    }

    fn token_url(&self) -> &str {
        "https://discord.com/api/oauth2/token"
    }

    fn default_scopes(&self) -> &'static [&'static str] {
        &["identify"]
    }

    fn user_info_url(&self) -> &str {
        "https://discord.com/api/v9/users/@me"
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
            email: user_info.email,
            email_verified: user_info.verified,
            name: user_info.global_name.or(Some(user_info.username)),
            avatar_url,
        })
    }
}
