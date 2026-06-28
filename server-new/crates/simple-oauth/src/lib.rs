use oauth2::{
    CsrfToken, HttpClientError, RequestTokenError, TokenResponse,
    basic::{BasicClient, BasicErrorResponse},
};

pub mod common;
mod provider;
pub mod types;

pub use provider::SimpleOAuthProvider;

use crate::types::{AuthorizeUrl, OAuthCredentials, StandardTokenResponse, UserInfo};

#[derive(Debug, thiserror::Error)]
pub enum SimpleOAuthError {
    #[error(transparent)]
    Request(#[from] reqwest::Error),
    #[error("invalid url: {0}")]
    ParseUrl(#[from] oauth2::url::ParseError),
    #[error("token exchange error: {0}")]
    TokenExchange(#[from] RequestTokenError<HttpClientError<reqwest::Error>, BasicErrorResponse>),
    #[error("deserialization error: {0}")]
    Deserialization(#[from] serde_json::Error),
}

pub struct SimpleOAuthClient {
    http_client: reqwest::Client,
    oauth_client: oauth2_reqwest::ReqwestClient,
}

impl SimpleOAuthClient {
    pub fn new() -> Result<Self, SimpleOAuthError> {
        let http_client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        Ok(Self {
            oauth_client: oauth2_reqwest::ReqwestClient::from(http_client.clone()),
            http_client,
        })
    }

    pub fn with_http_client(http_client: reqwest::Client) -> Self {
        Self {
            oauth_client: oauth2_reqwest::ReqwestClient::from(http_client.clone()),
            http_client,
        }
    }

    pub fn authorize_url<P: SimpleOAuthProvider + ?Sized>(
        &self,
        provider: &P,
        credentials: OAuthCredentials<'_>,
        redirect_url: &str,
        custom_scopes: Option<&[&str]>,
    ) -> Result<AuthorizeUrl, SimpleOAuthError> {
        let oauth_client =
            BasicClient::new(oauth2::ClientId::new(credentials.client_id.into_owned()))
                .set_client_secret(oauth2::ClientSecret::new(
                    credentials.client_secret.into_owned(),
                ))
                .set_auth_uri(oauth2::AuthUrl::new(provider.authorize_url().into())?)
                .set_redirect_uri(oauth2::RedirectUrl::new(redirect_url.into())?);
        let (pkce_challenge, pkce_verifier) = oauth2::PkceCodeChallenge::new_random_sha256();
        let (url, state) = oauth_client
            .authorize_url(CsrfToken::new_random)
            .add_scopes(
                custom_scopes
                    .unwrap_or(provider.default_scopes())
                    .into_iter()
                    .map(|s| oauth2::Scope::new((*s).to_owned())),
            )
            .set_pkce_challenge(pkce_challenge)
            .url();

        Ok(AuthorizeUrl {
            url,
            state: state.into_secret(),
            pkce_verifier: pkce_verifier.into_secret(),
        })
    }

    pub async fn exchange_code<P: SimpleOAuthProvider + ?Sized>(
        &self,
        provider: &P,
        credentials: OAuthCredentials<'_>,
        redirect_url: &str,
        code: &str,
        pkce_verifier: Option<&str>,
    ) -> Result<StandardTokenResponse, SimpleOAuthError> {
        let oauth_client =
            BasicClient::new(oauth2::ClientId::new(credentials.client_id.into_owned()))
                .set_client_secret(oauth2::ClientSecret::new(
                    credentials.client_secret.into_owned(),
                ))
                .set_redirect_uri(oauth2::RedirectUrl::new(redirect_url.into())?)
                .set_token_uri(oauth2::TokenUrl::new(provider.token_url().into())?);
        let mut token_request =
            oauth_client.exchange_code(oauth2::AuthorizationCode::new(code.into()));
        if let Some(verifier) = pkce_verifier {
            token_request =
                token_request.set_pkce_verifier(oauth2::PkceCodeVerifier::new(verifier.into()));
        }
        let token = token_request.request_async(&self.oauth_client).await?;

        Ok(StandardTokenResponse {
            access_token: token.access_token().secret().to_owned(),
            refresh_token: token.refresh_token().map(|s| s.secret().to_owned()),
            expires_in: token.expires_in(),
        })
    }

    pub async fn get_user_info<P: SimpleOAuthProvider + ?Sized>(
        &self,
        provider: &P,
        access_token: &str,
    ) -> Result<UserInfo, SimpleOAuthError> {
        let mut user_info_request = self
            .http_client
            .get(provider.user_info_url())
            .bearer_auth(access_token);
        for (name, val) in provider.create_request_headers() {
            user_info_request = user_info_request.header(name, val);
        }

        let user_info_val = user_info_request
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        let user_info = provider.extract_user_info(user_info_val)?;

        Ok(user_info)
    }
}
