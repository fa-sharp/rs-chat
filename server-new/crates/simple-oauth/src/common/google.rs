use serde::Deserialize;

use crate::{SimpleOAuthProvider, types::UserInfo};

pub struct Google {
    client_id: String,
    client_secret: String,
}

impl Google {
    pub fn new(client_id: impl Into<String>, client_secret: impl Into<String>) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
        }
    }
}

/// User info from Google API
#[derive(Debug, Deserialize)]
pub struct GoogleUserInfo {
    sub: String,
    name: String,
    picture: Option<String>,
}

impl SimpleOAuthProvider for Google {
    fn get_scopes(&self) -> Vec<String> {
        vec!["openid".into(), "profile".into()]
    }

    fn get_authorize_url(&self) -> String {
        "https://accounts.google.com/o/oauth2/v2/auth".into()
    }

    fn get_token_url(&self) -> String {
        "https://oauth2.googleapis.com/token".into()
    }

    fn get_user_info_url(&self) -> String {
        "https://www.googleapis.com/oauth2/v3/userinfo".into()
    }

    fn get_client_id(&self) -> String {
        self.client_id.clone()
    }

    fn get_client_secret(&self) -> String {
        self.client_secret.clone()
    }

    fn extract_user_info(&self, val: serde_json::Value) -> Result<UserInfo, serde_json::Error> {
        let user_info: GoogleUserInfo = serde_json::from_value(val)?;

        Ok(UserInfo {
            id: user_info.sub,
            name: user_info.name,
            avatar_url: user_info.picture,
        })
    }
}
