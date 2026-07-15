use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use simple_oauth::types::{OAuthCredentials, UserInfo};

use crate::{
    db::models::{ChatRsUser, NewChatRsUser, UpdateChatRsUser},
    services::auth::{AuthResult, oauth::OAuthProvider},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GoogleOAuthConfig {
    client_id: String,
    client_secret: String,
}

pub struct GoogleProvider {
    config: GoogleOAuthConfig,
}

impl GoogleProvider {
    pub fn new(config: &GoogleOAuthConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

impl OAuthProvider for GoogleProvider {
    fn get_inner_provider(&self) -> Box<dyn simple_oauth::SimpleOAuthProvider> {
        Box::new(simple_oauth::common::Google)
    }

    fn get_credentials(&self) -> OAuthCredentials {
        OAuthCredentials::new(&self.config.client_id, &self.config.client_secret)
    }

    fn find_linked_user<'a>(
        &self,
        db: &'a mut crate::db::DbService,
        user_data: &'a UserInfo,
    ) -> BoxFuture<'a, AuthResult<Option<ChatRsUser>>> {
        Box::pin(async move {
            let user = db.users().find_by_google_id(&user_data.id).await?;
            Ok(user)
        })
    }

    fn is_user_linked(&self, user: &ChatRsUser) -> bool {
        user.google_id.is_some()
    }

    fn create_update_user<'a>(&self, user_data: &'a UserInfo) -> UpdateChatRsUser<'a> {
        UpdateChatRsUser {
            google_id: Some(&user_data.id),
            ..Default::default()
        }
    }

    fn create_new_user<'a>(&self, user_data: &'a UserInfo) -> NewChatRsUser<'a> {
        NewChatRsUser {
            google_id: Some(&user_data.id),
            name: user_data
                .name
                .as_deref()
                .or(user_data.username.as_deref())
                .unwrap_or_default(),
            avatar_url: user_data.avatar_url.as_deref(),
            ..Default::default()
        }
    }
}
