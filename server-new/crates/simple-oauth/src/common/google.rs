use serde::Deserialize;

use crate::{SimpleOAuthProvider, types::UserInfo};

#[derive(Debug)]
pub struct Google;

/// User info from Google API
#[derive(Debug, Deserialize)]
struct GoogleUserInfo {
    sub: String,
    name: Option<String>,
    preferred_username: Option<String>,
    email: Option<String>,
    email_verified: Option<bool>,
    picture: Option<String>,
}

impl SimpleOAuthProvider for Google {
    fn default_scopes(&self) -> &'static [&'static str] {
        &["openid", "profile"]
    }

    fn authorize_url(&self) -> &str {
        "https://accounts.google.com/o/oauth2/v2/auth"
    }

    fn token_url(&self) -> &str {
        "https://oauth2.googleapis.com/token"
    }

    fn user_info_url(&self) -> &str {
        "https://www.googleapis.com/oauth2/v3/userinfo"
    }

    fn extract_user_info(&self, val: serde_json::Value) -> Result<UserInfo, serde_json::Error> {
        let user_info: GoogleUserInfo = serde_json::from_value(val)?;

        Ok(UserInfo {
            id: user_info.sub,
            name: user_info.name.or(user_info.preferred_username),
            email: user_info.email,
            email_verified: user_info.email_verified,
            avatar_url: user_info.picture,
        })
    }
}
