use anyhow::Context;
use oauth2::{StandardToken, Token};
use subtle::ConstantTimeEq;
use tower_sessions::Session;

use crate::{
    config::AppConfig,
    db::{DbPool, DbService, models::ChatRsUser},
    error::{AppError, AppResult},
    extractors::session::UserSession,
    services::auth::oauth::{OAuthProvider, OAuthProviderEnum},
};

/// OAuth functions
pub struct OAuthService<'a> {
    config: &'a AppConfig,
    db: &'a DbPool,
    http_client: &'a reqwest::Client,
}

impl<'a> OAuthService<'a> {
    const SESS_STATE_FIELD: &'static str = "oauth_state";
    const SESS_PKCE_FIELD: &'static str = "oauth_verifier";

    pub fn new(config: &'a AppConfig, db: &'a DbPool, http_client: &'a reqwest::Client) -> Self {
        Self {
            config,
            db,
            http_client,
        }
    }

    pub async fn authorize_url(
        &self,
        provider: OAuthProviderEnum,
        callback_path: &str,
        session: &Session,
    ) -> AppResult<oauth2::Url> {
        let provider = self.get_provider(provider)?;
        let client = self.get_oauth_client(provider.as_ref(), callback_path)?;

        let state = oauth2::State::new_random();
        let pkce_verifier = oauth2::PkceCodeVerifierS256::new_random();
        let mut auth_url = client.authorize_url(&state);
        auth_url
            .query_pairs_mut()
            .extend_pairs(pkce_verifier.authorize_url_params());

        session.insert(Self::SESS_STATE_FIELD, state).await?;
        session.insert(Self::SESS_PKCE_FIELD, pkce_verifier).await?;

        Ok(auth_url)
    }

    pub async fn exchange_code(
        &self,
        provider: OAuthProviderEnum,
        callback_path: &str,
        session: &Session,
        code: &str,
        returned_state: &str,
    ) -> AppResult<oauth2::StandardToken> {
        // Get saved state and code verifier from session
        let saved_state = session
            .remove::<oauth2::State>(Self::SESS_STATE_FIELD)
            .await?
            .ok_or_else(|| AppError::unauthorized("no state in session"))?;
        let pkce_verifier = session
            .remove::<oauth2::PkceCodeVerifierS256>(Self::SESS_PKCE_FIELD)
            .await?
            .ok_or_else(|| AppError::unauthorized("no PKCE verifier in session"))?;

        // Verify state
        if saved_state.ct_ne(returned_state.as_bytes()).into() {
            return Err(AppError::unauthorized("state parameter doesn't match"));
        }

        // Exchange code for token
        let provider = self.get_provider(provider)?;
        let client = self.get_oauth_client(provider.as_ref(), callback_path)?;
        let response = client
            .exchange_code(code)
            .param("code_verifier", String::from(pkce_verifier))
            .with_reqwest_client(&self.http_client)
            .execute::<StandardToken>()
            .await
            .map_err(|err| AppError::unauthorized(format!("token exchange failed: {err}")))?;

        Ok(response)
    }

    pub async fn get_user(
        &self,
        provider: OAuthProviderEnum,
        token: &oauth2::StandardToken,
        active_session: Option<UserSession>,
    ) -> AppResult<ChatRsUser> {
        let provider = self.get_provider(provider)?;
        let mut user_info_request = self
            .http_client
            .get(provider.get_user_info_url())
            .bearer_auth(token.access_token().as_ref());
        for (name, value) in provider.create_request_headers() {
            user_info_request = user_info_request.header(name, value);
        }
        let user_info_response = user_info_request.send().await.context("request failed")?;
        if !user_info_response.status().is_success() {
            let error =
                anyhow::anyhow!("failed to get user: {:?}", user_info_response.text().await);
            return Err(AppError::internal(error));
        }
        let user_data = provider
            .extract_user_data(user_info_response.json().await.context("request failed")?)
            .context("unable to extract user data from response")?;

        let mut db = DbService::from_pool(self.db).await?;
        let user = match provider.find_linked_user(&mut db, &user_data).await? {
            Some(existing_user) => {
                if active_session.is_some_and(|sess| sess.user_id != existing_user.id) {
                    return Err(AppError::unauthorized("cannot switch users via OAuth"));
                } else {
                    existing_user
                }
            }
            None => match active_session {
                None => {
                    let new_user = provider.create_new_user(&user_data);
                    db.users().create(new_user).await?
                }
                Some(sess) => match db.users().find_by_id(&sess.user_id).await? {
                    Some(user) if provider.is_user_linked(&user) => {
                        return Err(AppError::bad_request("user already linked to provider"));
                    }
                    Some(user) => {
                        // Link logged-in user to new provider
                        let update_user = provider.create_update_user(&user_data);
                        db.users().update(&user.id, update_user).await?;
                        user
                    }
                    None => {
                        return Err(AppError::internal(anyhow::anyhow!("user not found")));
                    }
                },
            },
        };

        Ok(user)
    }

    fn get_provider(&self, provider: OAuthProviderEnum) -> AppResult<Box<dyn OAuthProvider>> {
        let provider: Option<Box<dyn OAuthProvider>> = match provider {
            OAuthProviderEnum::Github => match self.config.auth.github {
                Some(ref c) => Some(Box::new(super::github::GitHubOAuthProvider::new(c))),
                None => None,
            },
            OAuthProviderEnum::Discord => match self.config.auth.discord {
                Some(ref c) => Some(Box::new(super::discord::DiscordOAuthProvider::new(c))),
                None => None,
            },
        };

        provider.ok_or_else(|| AppError::bad_request("unsupported OAuth provider"))
    }

    fn get_oauth_client(
        &self,
        provider: &dyn OAuthProvider,
        callback_path: &str,
    ) -> anyhow::Result<oauth2::Client> {
        let mut client = oauth2::Client::new(
            provider.get_client_id(),
            provider.get_authorize_url().parse()?,
            provider.get_token_url().parse()?,
        );
        client.set_client_secret(provider.get_client_secret());
        client.set_redirect_url(
            format!("{}{}", &self.config.server.base_url, callback_path).parse()?,
        );
        for scope in provider.get_scopes() {
            client.add_scope(scope);
        }

        Ok(client)
    }
}
