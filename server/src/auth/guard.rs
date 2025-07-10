use std::ops::Deref;

use rocket::{
    http::Status,
    outcome::try_outcome,
    request::{FromRequest, Outcome},
};
use rocket_flex_session::Session;
use rocket_okapi::OpenApiFromRequest;
use uuid::Uuid;

use crate::{
    auth::{
        sso_header::{get_sso_auth_outcome, get_sso_user_from_headers},
        ChatRsAuthSession, SSOHeaderMergedConfig,
    },
    db::{models::ChatRsUser, services::user::UserDbService, DbConnection},
};

/// User ID request guard to ensure a logged-in user.
#[derive(OpenApiFromRequest)]
pub struct ChatRsUserId(pub Uuid);
impl Deref for ChatRsUserId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Request guard / middleware to ensure a logged-in user.
#[rocket::async_trait]
impl<'r> FromRequest<'r> for ChatRsUserId {
    type Error = &'r str;

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        // Try authentication via proxy headers if configured
        if let Some(config) = req.rocket().state::<SSOHeaderMergedConfig>() {
            if let Some(proxy_user) = get_sso_user_from_headers(config, req.headers()) {
                let mut db = try_outcome!(req.guard::<DbConnection>().await);
                let mut db_service = UserDbService::new(&mut db);
                return get_sso_auth_outcome(&proxy_user, config, &mut db_service).await;
            } else {
                rocket::debug!("SSO header auth: headers not found");
            }
        };

        // Try authentication via session
        let session = req
            .guard::<Session<ChatRsAuthSession>>()
            .await
            .expect("should not fail");
        session.tap(|session| match session {
            Some(data) => Outcome::Success(ChatRsUserId(data.user_id)),
            None => Outcome::Error((Status::Unauthorized, "Unauthorized")),
        })
    }
}

/// Request guard / middleware to get the current user data from the database.
#[rocket::async_trait]
impl<'r> FromRequest<'r> for ChatRsUser {
    type Error = &'r str;

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let mut db = try_outcome!(req.guard::<DbConnection>().await);
        let user_id = try_outcome!(req.guard::<ChatRsUserId>().await);

        match UserDbService::new(&mut db).find_by_id(&user_id).await {
            Ok(Some(user)) => Outcome::Success(user),
            Ok(None) => Outcome::Error((Status::NotFound, "User not found")),
            Err(e) => {
                rocket::error!("Session guard: database error: {}", e);
                Outcome::Error((Status::InternalServerError, "Server error"))
            }
        }
    }
}
