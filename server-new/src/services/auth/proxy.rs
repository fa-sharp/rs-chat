use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    db::{
        DbService,
        models::{ChatRsUser, NewChatRsUser},
    },
    services::auth::error::{AuthError, AuthResult},
};

/// SSO / forward auth proxy header configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyHeaderConfig {
    /// Whether proxy header authentication is enabled
    pub enabled: bool,
    /// Header for unique, identifying username (default: `Remote-User`)
    username_header: String,
    /// Header for display name (default: `Remote-Name`)
    name_header: String,
    /// Header for space-delimited groups/roles of the user (default: `Remote-Groups`)
    groups_header: String,
    /// If set, only users in these groups will be allowed to access the app
    user_groups: Option<Vec<String>>,
    /// URL to redirect to in order to log out of the remote service
    logout_url: Option<String>,
}

impl Default for ProxyHeaderConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            username_header: String::from("Remote-User"),
            name_header: String::from("Remote-Name"),
            groups_header: String::from("Remote-Groups"),
            user_groups: None,
            logout_url: None,
        }
    }
}

pub struct ProxyService<'r> {
    config: &'r ProxyHeaderConfig,
}

pub struct ProxyUser {
    username: String,
    name: String,
}

impl<'r> ProxyService<'r> {
    pub fn new(config: &'r ProxyHeaderConfig) -> Self {
        Self { config }
    }

    pub fn extract_proxy_user(&self, headers: &HeaderMap) -> AuthResult<Option<ProxyUser>> {
        let Some(username) = headers.get(&self.config.username_header) else {
            return Ok(None);
        };
        let name = headers.get(&self.config.name_header).unwrap_or(username);
        let groups = headers
            .get(&self.config.groups_header)
            .and_then(|groups| groups.to_str().ok())
            .unwrap_or_default();

        if let Some(ref allowed_groups) = self.config.user_groups
            && !is_proxy_user_allowed(groups, allowed_groups)
        {
            return Err(AuthError::Unauthorized("proxy user not in allowed group"));
        }

        Ok(Some(ProxyUser {
            username: username.to_str().unwrap_or_default().to_owned(),
            name: name.to_str().unwrap_or_default().to_owned(),
        }))
    }

    pub async fn find_proxy_user(
        &self,
        db: &mut DbService,
        proxy_user: &ProxyUser,
    ) -> AuthResult<Option<Uuid>> {
        let user_id = db
            .users()
            .find_by_sso_username(&proxy_user.username)
            .await?;
        Ok(user_id)
    }

    pub async fn create_proxy_user(
        &self,
        db: &mut DbService,
        proxy_user: &ProxyUser,
    ) -> AuthResult<ChatRsUser> {
        let new_user = db
            .users()
            .create(NewChatRsUser {
                sso_username: Some(&proxy_user.username),
                name: &proxy_user.name,
                ..Default::default()
            })
            .await?;
        Ok(new_user)
    }
}

fn is_proxy_user_allowed(user_groups: &str, allowed_groups: &[String]) -> bool {
    for user_group in user_groups.split(' ') {
        if allowed_groups.iter().any(|g| g == user_group) {
            return true;
        }
    }

    false
}
