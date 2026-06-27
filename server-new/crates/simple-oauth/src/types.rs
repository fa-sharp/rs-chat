pub struct AuthorizeUrl {
    pub url: oauth2::url::Url,
    pub state: String,
    pub pkce_verifier: String,
}

pub struct StandardTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<std::time::Duration>,
}

pub struct UserInfo {
    pub id: String,
    pub name: String,
    pub avatar_url: Option<String>,
}
