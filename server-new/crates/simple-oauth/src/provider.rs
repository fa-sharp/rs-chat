use std::fmt::Debug;

use crate::types::UserInfo;

/// Trait for all OAuth providers
pub trait SimpleOAuthProvider: Debug + Send + Sync {
    fn authorize_url(&self) -> &str;
    fn token_url(&self) -> &str;
    fn user_info_url(&self) -> &str;
    fn default_scopes(&self) -> &'static [&'static str];
    fn create_request_headers(&self) -> Vec<(String, String)> {
        vec![]
    }
    fn extract_user_info(&self, val: serde_json::Value) -> Result<UserInfo, serde_json::Error>;
}
