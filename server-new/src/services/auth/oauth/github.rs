use futures::future::BoxFuture;
use serde::Deserialize;
use simple_oauth::{
    SimpleOAuthProvider,
    types::{OAuthCredentials, UserInfo},
};

use crate::{
    db::models::{ChatRsUser, NewChatRsUser, UpdateChatRsUser},
    services::auth::{AuthResult, oauth::OAuthProvider},
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
    fn get_inner_provider(&self) -> Box<dyn SimpleOAuthProvider> {
        Box::new(simple_oauth::common::github::GitHub)
    }

    fn get_credentials(&self) -> OAuthCredentials<'_> {
        OAuthCredentials::new(&self.config.client_id, &self.config.client_secret)
    }

    fn find_linked_user<'a>(
        &self,
        db: &'a mut crate::db::DbService,
        user_data: &'a UserInfo,
    ) -> BoxFuture<'a, AuthResult<Option<ChatRsUser>>> {
        Box::pin(async move {
            let user = db.users().find_by_github_id(&user_data.id).await?;
            Ok(user)
        })
    }

    fn is_user_linked(&self, user: &ChatRsUser) -> bool {
        user.github_id.is_some()
    }

    fn create_update_user<'a>(&self, user_data: &'a UserInfo) -> UpdateChatRsUser<'a> {
        UpdateChatRsUser {
            github_id: Some(&user_data.id),
            ..Default::default()
        }
    }

    fn create_new_user<'a>(&self, user_data: &'a UserInfo) -> NewChatRsUser<'a> {
        NewChatRsUser {
            github_id: Some(&user_data.id),
            name: &user_data.name.as_deref().unwrap_or_default(),
            avatar_url: user_data.avatar_url.as_deref(),
            ..Default::default()
        }
    }
}
