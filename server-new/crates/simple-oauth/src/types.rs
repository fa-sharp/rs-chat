use std::{borrow::Cow, fmt::Debug};

use serde::Deserialize;

/// OAuth2 authorization redirect URL, along with the state and PKCE verifier
pub struct AuthorizeUrl {
    pub url: oauth2::url::Url,
    pub state: String,
    pub pkce_verifier: String,
}
impl Debug for AuthorizeUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthorizeUrl")
            .field("url", &"--redacted--")
            .field("state", &self.state)
            .field("pkce_verifier", &"--redacted--")
            .finish()
    }
}

/// User info returned by the OAuth provider
#[derive(Debug)]
pub struct UserInfo {
    /// The ID of the user at the OAuth provider
    pub id: String,
    /// The user's display name
    pub name: Option<String>,
    /// The user's email. Likely will not be included unless you add the proper email scope for the provider.
    ///
    /// ⚠️ Do not rely on this for identifying the user. Use the `id` and the provider name.
    pub email: Option<String>,
    /// Whether the user's email is verified. Not all providers return this in the user info.
    pub email_verified: Option<bool>,
    /// The URL of the user's picture/avatar
    pub avatar_url: Option<String>,
}

/// Standard OAuth2 token response
pub struct StandardTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<std::time::Duration>,
}
impl Debug for StandardTokenResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StandardTokenResponse")
            .field("access_token", &"--redacted--")
            .field("refresh_token", &"--redacted--")
            .field("expires_in", &self.expires_in)
            .finish()
    }
}

pub struct OAuthCredentials<'a> {
    pub client_id: Cow<'a, str>,
    pub client_secret: Cow<'a, str>,
}
impl<'a> OAuthCredentials<'a> {
    pub fn new(client_id: impl Into<Cow<'a, str>>, client_secret: impl Into<Cow<'a, str>>) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
        }
    }
}
impl<'a> Debug for OAuthCredentials<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OAuthCredentials")
            .field("client_id", &self.client_id)
            .field("client_secret", &"--redacted--")
            .finish()
    }
}

/// OIDC discovery document
#[derive(Debug, Default, Deserialize)]
pub struct OidcDiscovery {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub scopes_supported: Option<Vec<String>>,
}
