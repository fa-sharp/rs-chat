use std::ops::Deref;

use rocket::{
    http::Status,
    outcome::try_outcome,
    request::{FromRequest, Outcome},
};
use rocket_flex_session::Session;
use rocket_okapi::{
    gen::OpenApiGenerator,
    okapi::openapi3,
    request::{OpenApiFromRequest, RequestHeaderInput},
};
use uuid::Uuid;

use crate::{
    auth::{
        api_key::get_api_key_auth_outcome,
        sso_header::{get_sso_auth_outcome, get_sso_user_from_headers},
        ChatRsAuthSession, SSOHeaderMergedConfig,
    },
    db::{models::ChatRsUser, services::UserDbService, DbConnection},
    utils::Encryptor,
};

/// User ID request guard to ensure a logged-in user.
pub struct ChatRsUserId(pub(super) Uuid);

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
                return get_sso_auth_outcome(&proxy_user, config, &mut db).await;
            }
        };

        // Try authentication via API key
        if let Some(auth_header) = req.headers().get_one("Authorization") {
            let encryptor = req.rocket().state::<Encryptor>().expect("should exist");
            let mut db = try_outcome!(req.guard::<DbConnection>().await);
            return get_api_key_auth_outcome(auth_header, encryptor, &mut db).await;
        }

        // Try authentication via session
        let session = try_outcome!(req.guard::<Session<ChatRsAuthSession>>().await);
        match session.tap(|data| data.and_then(|auth_session| auth_session.user_id())) {
            Some(user_id) => Outcome::Success(ChatRsUserId(user_id)),
            None => Outcome::Error((Status::Unauthorized, "Unauthorized")),
        }
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
                rocket::error!("User guard: database error: {}", e);
                Outcome::Error((Status::InternalServerError, "Database error"))
            }
        }
    }
}

/// OpenAPI documentation for API key authentication when using the ChatRsUserId guard.
impl<'a> OpenApiFromRequest<'a> for ChatRsUserId {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        api_key_docs()
    }
}

/// OpenAPI documentation for API key authentication when using the ChatRsUser guard.
impl<'a> OpenApiFromRequest<'a> for ChatRsUser {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        api_key_docs()
    }
}

fn api_key_docs() -> Result<RequestHeaderInput, rocket_okapi::OpenApiError> {
    let security_scheme = openapi3::SecurityScheme {
        description: Some("Requires an API key to access.".to_owned()),
        data: openapi3::SecuritySchemeData::Http {
            scheme: "bearer".to_owned(),
            bearer_format: Some("bearer".to_owned()),
        },
        extensions: openapi3::Object::default(),
    };
    let mut security_req = openapi3::SecurityRequirement::new();
    security_req.insert("API Key".to_owned(), Vec::new());
    Ok(RequestHeaderInput::Security(
        "API Key".to_owned(),
        security_scheme,
        security_req,
    ))
}
