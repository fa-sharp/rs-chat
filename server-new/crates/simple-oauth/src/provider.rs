use crate::types::UserInfo;

/// Trait for all OAuth providers
pub trait SimpleOAuthProvider: Send + Sync {
    fn get_scopes(&self) -> Vec<String>;
    fn get_authorize_url(&self) -> String;
    fn get_token_url(&self) -> String;
    fn get_user_info_url(&self) -> String;
    fn get_client_id(&self) -> String;
    fn get_client_secret(&self) -> String;
    fn create_request_headers(&self) -> Vec<(String, String)> {
        vec![]
    }
    fn extract_user_info(
        &self,
        user_info: serde_json::Value,
    ) -> Result<UserInfo, serde_json::Error>;
}
