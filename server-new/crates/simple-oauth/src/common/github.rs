use serde::Deserialize;

use crate::{SimpleOAuthProvider, types::UserInfo};

#[derive(Debug)]
pub struct GitHub;

/// User info returned from GitHub API
#[derive(Debug, Deserialize)]
struct GitHubUserInfo {
    id: u64,
    login: String,
    name: Option<String>,
    email: Option<String>,
    avatar_url: Option<String>,
}

impl SimpleOAuthProvider for GitHub {
    fn authorize_url(&self) -> &str {
        "https://github.com/login/oauth/authorize"
    }

    fn token_url(&self) -> &str {
        "https://github.com/login/oauth/access_token"
    }

    fn default_scopes(&self) -> &'static [&'static str] {
        &["read:user"]
    }

    fn user_info_url(&self) -> &str {
        "https://api.github.com/user"
    }

    fn create_request_headers(&self) -> Vec<(String, String)> {
        vec![
            ("Accept".into(), "application/vnd.github+json".into()),
            ("User-Agent".into(), "fa-sharp/simple-oauth".into()),
        ]
    }

    fn extract_user_info(
        &self,
        user_info: serde_json::Value,
    ) -> Result<UserInfo, serde_json::Error> {
        let info: GitHubUserInfo = serde_json::from_value(user_info)?;

        Ok(UserInfo {
            id: info.id.to_string(),
            name: info.name.or(Some(info.login)),
            email: info.email,
            email_verified: None,
            avatar_url: info.avatar_url,
        })
    }
}
