use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};

use crate::{
    db::{
        DbService,
        models::{ChatRsUser, NewChatRsUser, UpdateChatRsUser},
    },
    error::AppResult,
};

mod discord;
mod github;
mod service;

pub use discord::DiscordOAuthConfig;
pub use github::GitHubOAuthConfig;
pub use service::OAuthService;

/// Supported OAuth providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OAuthProviderEnum {
    Github,
    Discord,
}
impl OAuthProviderEnum {
    pub fn as_str(&self) -> &str {
        match self {
            OAuthProviderEnum::Github => "github",
            OAuthProviderEnum::Discord => "discord",
        }
    }
}

/// Trait for all OAuth providers
pub trait OAuthProvider: Send + Sync {
    fn get_scopes(&self) -> Vec<&str>;
    fn get_authorize_url(&self) -> &str;
    fn get_token_url(&self) -> &str;
    fn get_user_info_url(&self) -> &str;
    fn get_client_id(&self) -> String;
    fn get_client_secret(&self) -> String;
    fn create_request_headers(&self) -> Vec<(&'static str, &'static str)> {
        vec![]
    }
    fn extract_user_data(&self, user_info: serde_json::Value) -> anyhow::Result<UserData>;
    fn find_linked_user<'a>(
        &self,
        db: &'a mut DbService,
        user_data: &'a UserData,
    ) -> BoxFuture<'a, AppResult<Option<ChatRsUser>>>;
    fn is_user_linked(&self, user: &ChatRsUser) -> bool;
    fn create_update_user<'a>(&self, user_data: &'a UserData) -> UpdateChatRsUser<'a>;
    fn create_new_user<'a>(&self, user_data: &'a UserData) -> NewChatRsUser<'a>;
}

/// Common OAuth user data structure
#[derive(Debug)]
pub struct UserData {
    pub id: String,
    pub name: String,
    pub avatar_url: Option<String>,
}
