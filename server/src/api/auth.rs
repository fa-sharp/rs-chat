use rocket::{get, Route};
use rocket_flex_session::Session;
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
};
use uuid::Uuid;

use crate::{auth::ChatRsAuthSession, errors::ApiError};

pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: login, user, logout]
}

#[openapi]
#[get("/login")]
async fn login(mut session: Session<'_, ChatRsAuthSession>) -> Result<String, ApiError> {
    session.set(ChatRsAuthSession::new(Uuid::new_v4()));

    Ok("Login successful".to_string())
}

#[openapi]
#[get("/user")]
async fn user(session: Session<'_, ChatRsAuthSession>) -> Result<String, ApiError> {
    let user_id = session
        .tap(|session| session.map(|s| s.user_id))
        .ok_or(ApiError::Authentication("No session found".into()))?;

    Ok(format!("User ID: {}", user_id))
}

#[openapi]
#[get("/logout")]
async fn logout(mut session: Session<'_, ChatRsAuthSession>) -> Result<String, ApiError> {
    session.delete();

    Ok("Logout successful".to_string())
}
