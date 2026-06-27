use serde::Deserialize;

use crate::{SimpleOAuthProvider, types::UserInfo};

pub struct GitHub {
    client_id: String,
    client_secret: String,
    user_agent: String,
}

impl GitHub {
    pub fn new(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        user_agent: impl Into<String>,
    ) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            user_agent: user_agent.into(),
        }
    }
}

/// User info returned from GitHub API
#[derive(Debug, Deserialize)]
struct GitHubUserInfo {
    id: u64,
    login: String,
    name: Option<String>,
    avatar_url: Option<String>,
}

impl SimpleOAuthProvider for GitHub {
    fn get_authorize_url(&self) -> String {
        String::from("https://github.com/login/oauth/authorize")
    }

    fn get_token_url(&self) -> String {
        String::from("https://github.com/login/oauth/access_token")
    }

    fn get_scopes(&self) -> Vec<String> {
        vec!["user:read".into()]
    }

    fn get_user_info_url(&self) -> String {
        String::from("https://api.github.com/user")
    }

    fn get_client_id(&self) -> String {
        self.client_id.to_owned()
    }

    fn get_client_secret(&self) -> String {
        self.client_secret.to_owned()
    }

    fn create_request_headers(&self) -> Vec<(String, String)> {
        vec![
            ("Accept".into(), "application/vnd.github+json".into()),
            ("User-Agent".into(), self.user_agent.clone()),
        ]
    }

    fn extract_user_info(
        &self,
        user_info: serde_json::Value,
    ) -> Result<UserInfo, serde_json::Error> {
        let info: GitHubUserInfo = serde_json::from_value(user_info)?;

        Ok(UserInfo {
            id: info.id.to_string(),
            name: info.name.unwrap_or(info.login),
            avatar_url: info.avatar_url,
        })
    }
}
