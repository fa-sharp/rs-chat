use futures::future::BoxFuture;
use serde::Deserialize;
use simple_oauth::{common::discord::Discord, types::UserInfo};

use crate::{
    db::models::{ChatRsUser, NewChatRsUser, UpdateChatRsUser},
    services::auth::{AuthResult, oauth::OAuthProvider},
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
    fn get_inner_provider(&self) -> Box<dyn simple_oauth::SimpleOAuthProvider> {
        Box::new(Discord::new(
            self.config.client_id,
            &self.config.client_secret,
        ))
    }

    fn find_linked_user<'a>(
        &self,
        db: &'a mut crate::db::DbService,
        user_data: &'a UserInfo,
    ) -> BoxFuture<'a, AuthResult<Option<ChatRsUser>>> {
        Box::pin(async move {
            let user = db.users().find_by_discord_id(&user_data.id).await?;
            Ok(user)
        })
    }

    fn is_user_linked(&self, user: &ChatRsUser) -> bool {
        user.discord_id.is_some()
    }

    fn create_update_user<'a>(&self, user_data: &'a UserInfo) -> UpdateChatRsUser<'a> {
        UpdateChatRsUser {
            discord_id: Some(&user_data.id),
            ..Default::default()
        }
    }

    fn create_new_user<'a>(&self, user_data: &'a UserInfo) -> NewChatRsUser<'a> {
        NewChatRsUser {
            discord_id: Some(&user_data.id),
            name: &user_data.name,
            avatar_url: user_data.avatar_url.as_deref(),
            ..Default::default()
        }
    }
}
