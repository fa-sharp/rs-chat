use futures::future::BoxFuture;
use serde::Deserialize;

use crate::{
    db::models::{ChatRsUser, NewChatRsUser, UpdateChatRsUser},
    error::AppResult,
    services::auth::oauth::OAuthProvider,
};

#[derive(Clone, Debug, Deserialize)]
pub struct GitHubOAuthConfig {
    client_id: String,
    client_secret: String,
}

pub struct GitHubOAuthProvider {
    config: GitHubOAuthConfig,
}

impl GitHubOAuthProvider {
    pub fn new(config: &GitHubOAuthConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

impl OAuthProvider for GitHubOAuthProvider {
    fn get_authorize_url(&self) -> &str {
        "https://github.com/login/oauth/authorize"
    }

    fn get_token_url(&self) -> &str {
        "https://github.com/login/oauth/access_token"
    }

    fn get_scopes(&self) -> Vec<&str> {
        vec!["user:read"]
    }

    fn get_user_info_url(&self) -> &str {
        "https://api.github.com/user"
    }

    fn get_client_id(&self) -> String {
        self.config.client_id.to_owned()
    }

    fn get_client_secret(&self) -> String {
        self.config.client_secret.to_owned()
    }

    fn create_request_headers(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("Accept", "application/vnd.github+json"),
            ("User-Agent", "fa-sharp/rs-chat"),
        ]
    }

    fn extract_user_data(&self, user_info: serde_json::Value) -> anyhow::Result<super::UserData> {
        let info: GitHubUserInfo = serde_json::from_value(user_info)?;

        Ok(super::UserData {
            id: info.id.to_string(),
            name: info.name.unwrap_or(info.login),
            avatar_url: info.avatar_url,
        })
    }

    fn find_linked_user<'a>(
        &self,
        db: &'a mut crate::db::DbService,
        user_data: &'a super::UserData,
    ) -> BoxFuture<'a, AppResult<Option<ChatRsUser>>> {
        Box::pin(async move {
            let user = db.users().find_by_github_id(&user_data.id).await?;
            Ok(user)
        })
    }

    fn is_user_linked(&self, user: &ChatRsUser) -> bool {
        user.github_id.is_some()
    }

    fn create_update_user<'a>(&self, user_data: &'a super::UserData) -> UpdateChatRsUser<'a> {
        UpdateChatRsUser {
            github_id: Some(&user_data.id),
            ..Default::default()
        }
    }

    fn create_new_user<'a>(&self, user_data: &'a super::UserData) -> NewChatRsUser<'a> {
        NewChatRsUser {
            github_id: Some(&user_data.id),
            name: &user_data.name,
            avatar_url: user_data.avatar_url.as_deref(),
            ..Default::default()
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
