mod oauth;
mod session;
mod sso_header;

pub use oauth::{setup_oauth, DiscordOAuthConfig, GitHubOAuthConfig, GoogleOAuthConfig};
pub use session::{setup_session, ChatRsAuthSession};
pub use sso_header::{setup_sso_header_auth, SSOHeaderMergedConfig};

use rocket::{
    fairing::AdHoc,
    http::Status,
    request::{FromRequest, Outcome},
};
use rocket_flex_session::Session;

use crate::{
    auth::sso_header::{get_sso_auth_outcome, get_sso_user_from_headers},
    config::get_app_config,
    db::{models::ChatRsUser, services::user::UserDbService, DbConnection},
    utils::encryption::Encryptor,
};

/// Request guard / middleware to ensure a logged-in user.
#[rocket::async_trait]
impl<'r> FromRequest<'r> for ChatRsUser {
    type Error = &'r str;

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let Outcome::Success(mut db) = req.guard::<DbConnection>().await else {
            rocket::error!("Session guard: database connection failed");
            return Outcome::Error((Status::InternalServerError, "Server error"));
        };

        let mut db_service = UserDbService::new(&mut db);

        // Try authentication via proxy headers if configured
        if let Some(config) = req.rocket().state::<SSOHeaderMergedConfig>() {
            match get_sso_user_from_headers(config, req.headers()) {
                Some(proxy_user) => {
                    return get_sso_auth_outcome(&proxy_user, config, &mut db_service).await;
                }
                None => {
                    rocket::debug!("Proxy header auth: headers not found")
                }
            }
        };

        // Try authentication via session
        let session = req
            .guard::<Session<ChatRsAuthSession>>()
            .await
            .expect("should not fail");

        let Some(user_id) = session.tap(|session| match session {
            Some(data) => Some(data.user_id),
            None => None,
        }) else {
            return Outcome::Error((Status::Unauthorized, "Unauthorized"));
        };

        let user = db_service.find_by_id(&user_id).await;
        match user {
            Ok(Some(user)) => Outcome::Success(user),
            Ok(None) => Outcome::Error((Status::NotFound, "User not found")),
            Err(e) => {
                rocket::error!("Session guard: database error: {}", e);
                Outcome::Error((Status::InternalServerError, "Server error"))
            }
        }
    }
}

/// Fairing that sets up an encryption service
pub fn setup_encryption() -> AdHoc {
    AdHoc::on_ignite("Encryption setup", |rocket| async {
        let app_config = get_app_config(&rocket);
        let encryptor = Encryptor::new(&app_config.secret_key)
            .expect("Invalid secret key: must be 64-character hexadecimal string");

        rocket.manage(encryptor)
    })
}
