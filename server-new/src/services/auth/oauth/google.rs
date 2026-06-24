use futures::future::BoxFuture;
use serde::Deserialize;

use crate::{
    db::models::{ChatRsUser, NewChatRsUser, UpdateChatRsUser},
    services::auth::{
        AuthError, AuthResult,
        oauth::{OAuthProvider, UserData},
    },
};

#[derive(Clone, Debug, Deserialize)]
pub struct GoogleOAuthConfig {
    client_id: String,
    client_secret: String,
}

/// User info from Google API
#[derive(Debug, Deserialize)]
pub struct GoogleUserInfo {
    sub: String,
    name: String,
    picture: Option<String>,
}

pub struct GoogleOAuthProvider {
    config: GoogleOAuthConfig,
}

impl GoogleOAuthProvider {
    pub fn new(config: &GoogleOAuthConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

impl OAuthProvider for GoogleOAuthProvider {
    fn get_scopes(&self) -> Vec<&str> {
        vec!["openid", "profile"]
    }

    fn get_authorize_url(&self) -> &str {
        "https://accounts.google.com/o/oauth2/v2/auth"
    }

    fn get_token_url(&self) -> &str {
        "https://oauth2.googleapis.com/token"
    }

    fn get_user_info_url(&self) -> &str {
        "https://www.googleapis.com/oauth2/v3/userinfo"
    }

    fn get_client_id(&self) -> String {
        self.config.client_id.clone()
    }

    fn get_client_secret(&self) -> String {
        self.config.client_secret.clone()
    }

    fn extract_user_data(&self, user_info: serde_json::Value) -> AuthResult<UserData> {
        let user_info: GoogleUserInfo = serde_json::from_value(user_info).map_err(|err| {
            AuthError::Provider(anyhow::Error::from(err).context("parse user info"))
        })?;

        Ok(UserData {
            id: user_info.sub,
            name: user_info.name,
            avatar_url: user_info.picture,
        })
    }

    fn find_linked_user<'a>(
        &self,
        db: &'a mut crate::db::DbService,
        user_data: &'a super::UserData,
    ) -> BoxFuture<'a, AuthResult<Option<ChatRsUser>>> {
        Box::pin(async move {
            let user = db.users().find_by_google_id(&user_data.id).await?;
            Ok(user)
        })
    }

    fn is_user_linked(&self, user: &ChatRsUser) -> bool {
        user.google_id.is_some()
    }

    fn create_update_user<'a>(&self, user_data: &'a super::UserData) -> UpdateChatRsUser<'a> {
        UpdateChatRsUser {
            google_id: Some(&user_data.id),
            ..Default::default()
        }
    }

    fn create_new_user<'a>(&self, user_data: &'a super::UserData) -> NewChatRsUser<'a> {
        NewChatRsUser {
            google_id: Some(&user_data.id),
            name: &user_data.name,
            avatar_url: user_data.avatar_url.as_deref(),
            ..Default::default()
        }
    }
}
