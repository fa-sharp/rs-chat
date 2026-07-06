use aide::OperationIo;
use axum::extract::FromRequestParts;
use schemars::JsonSchema;
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::state::AppState;

/// The current auth configuration of the server
#[skip_serializing_none]
#[derive(Debug, Serialize, JsonSchema, OperationIo)]
pub struct PublicAuthConfig {
    /// Whether GitHub login is enabled
    github: bool,
    /// Whether Google login is enabled
    google: bool,
    /// Whether Discord login is enabled
    discord: bool,
    /// OIDC configuration
    oidc: Option<Oidc>,
    // /// SSO configuration
    // sso: Option<SSO>,
}

#[derive(Debug, Serialize, JsonSchema)]
struct Oidc {
    /// The name of the OIDC provider
    name: String,
}

// #[derive(Debug, JsonSchema, serde::Serialize)]
// struct SSO {
//     /// Whether SSO header authentication is enabled
//     enabled: bool,
//     /// The URL to redirect to after logout
//     logout_url: Option<String>,
// }

impl FromRequestParts<AppState> for PublicAuthConfig {
    type Rejection = ();

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(PublicAuthConfig {
            github: state.config.auth.github.is_some(),
            discord: state.config.auth.discord.is_some(),
            google: state.config.auth.google.is_some(),
            oidc: state.config.auth.oidc.as_ref().map(|oidc| Oidc {
                name: oidc.name.as_deref().unwrap_or("OIDC").to_owned(),
            }),
        })
    }
}
