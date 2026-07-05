use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use simple_oauth::{
    SimpleOAuthProvider,
    types::{OAuthCredentials, OidcDiscovery},
};

use crate::{
    db::models::{ChatRsUser, NewChatRsUser, UpdateChatRsUser},
    services::auth::AuthResult,
};

use super::OAuthProvider;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OidcConfig {
    pub name: Option<String>,
    client_id: String,
    client_secret: String,
    auth_endpoint: String,
    token_endpoint: String,
    userinfo_endpoint: String,
}

pub struct OidcProvider {
    config: OidcConfig,
}

impl OidcProvider {
    pub fn new(config: &OidcConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

impl OAuthProvider for OidcProvider {
    fn get_inner_provider(&self) -> Box<dyn SimpleOAuthProvider> {
        Box::new(simple_oauth::common::Oidc::from_config(OidcDiscovery {
            authorization_endpoint: self.config.auth_endpoint.clone(),
            token_endpoint: self.config.token_endpoint.clone(),
            userinfo_endpoint: self.config.userinfo_endpoint.clone(),
            ..Default::default()
        }))
    }

    fn get_credentials(&self) -> OAuthCredentials {
        OAuthCredentials::new(&self.config.client_id, &self.config.client_secret)
    }

    fn find_linked_user<'a>(
        &self,
        db: &'a mut crate::db::DbService,
        user_info: &'a simple_oauth::types::UserInfo,
    ) -> BoxFuture<'a, AuthResult<Option<ChatRsUser>>> {
        Box::pin(async move {
            let user = db.users().find_by_oidc_id(&user_info.id).await?;
            Ok(user)
        })
    }

    fn is_user_linked(&self, user: &ChatRsUser) -> bool {
        user.oidc_id.is_some()
    }

    fn create_update_user<'a>(
        &self,
        user_info: &'a simple_oauth::types::UserInfo,
    ) -> crate::db::models::UpdateChatRsUser<'a> {
        UpdateChatRsUser {
            oidc_id: Some(&user_info.id),
            ..Default::default()
        }
    }

    fn create_new_user<'a>(
        &self,
        user_info: &'a simple_oauth::types::UserInfo,
    ) -> crate::db::models::NewChatRsUser<'a> {
        NewChatRsUser {
            google_id: Some(&user_info.id),
            name: &user_info
                .name
                .as_deref()
                .or(user_info.username.as_deref())
                .unwrap_or_default(),
            avatar_url: user_info.avatar_url.as_deref(),
            ..Default::default()
        }
    }
}
