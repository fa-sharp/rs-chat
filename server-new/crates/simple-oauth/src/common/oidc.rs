use serde::Deserialize;

use crate::{
    SimpleOAuthError, SimpleOAuthProvider,
    types::{OidcDiscovery, UserInfo},
};

#[derive(Debug)]
pub struct Oidc {
    auth_endpoint: String,
    token_endpoint: String,
    userinfo_endpoint: String,
}

/// Standard OIDC user info shape
#[derive(Debug, Deserialize)]
struct OidcUserInfo {
    sub: String,
    name: Option<String>,
    preferred_username: Option<String>,
    email: Option<String>,
    email_verified: Option<bool>,
    picture: Option<String>,
}

impl Oidc {
    pub fn from_config(config: OidcDiscovery) -> Self {
        Self {
            auth_endpoint: config.authorization_endpoint,
            token_endpoint: config.token_endpoint,
            userinfo_endpoint: config.userinfo_endpoint,
        }
    }

    /// Discover the OIDC config from the given URL. This will fail
    /// if the discovery document is missing a token or userinfo endpoint.
    pub async fn discover(
        http_client: &reqwest::Client,
        discovery_url: &str,
    ) -> Result<Self, SimpleOAuthError> {
        let discovery = http_client
            .get(discovery_url)
            .send()
            .await?
            .error_for_status()?
            .json::<OidcDiscovery>()
            .await?;

        Ok(Self {
            auth_endpoint: discovery.authorization_endpoint,
            token_endpoint: discovery.token_endpoint,
            userinfo_endpoint: discovery.userinfo_endpoint,
        })
    }
}

impl SimpleOAuthProvider for Oidc {
    fn authorize_url(&self) -> &str {
        &self.auth_endpoint
    }

    fn token_url(&self) -> &str {
        &self.token_endpoint
    }

    fn user_info_url(&self) -> &str {
        &self.userinfo_endpoint
    }

    fn default_scopes(&self) -> &'static [&'static str] {
        &["openid", "profile"]
    }

    fn extract_user_info(&self, val: serde_json::Value) -> Result<UserInfo, serde_json::Error> {
        let user_info: OidcUserInfo = serde_json::from_value(val)?;

        Ok(UserInfo {
            id: user_info.sub,
            name: user_info.name.or(user_info.preferred_username),
            email: user_info.email,
            email_verified: user_info.email_verified,
            avatar_url: user_info.picture,
        })
    }
}
