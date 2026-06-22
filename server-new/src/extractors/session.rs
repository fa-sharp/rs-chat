use anyhow::Context;
use axum::extract::{FromRequestParts, OptionalFromRequestParts};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

use crate::error::AppError;

/// The field in the `tower_sessions` storage used to store the user session data
const USER_SESSION_FIELD: &str = "sess";

/// Active session data. Beware when changing or adding to this struct, as it can
/// invalidate existing sessions.
///
/// This can be used as an extractor in route handlers:
/// - If used as `Option<UserSession>`, will be `Some` if there is an active session
/// and `None` otherwise.
/// - If used as `UserSession`, request will automatically return an unauthorized error
/// if there is no active session.
#[derive(Debug, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: String,
}

impl UserSession {
    pub fn new(user_id: String) -> Self {
        Self { user_id }
    }

    pub async fn init(session: &Session, user_id: &str) -> Result<(), AppError> {
        Ok(session
            .insert(USER_SESSION_FIELD, UserSession::new(user_id.into()))
            .await
            .context("failed to initialize session")?)
    }
}

impl<S: Send + Sync> OptionalFromRequestParts<S> for UserSession {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|(_, msg)| AppError::internal(anyhow::anyhow!(msg)))?;

        match session.get::<UserSession>(USER_SESSION_FIELD).await {
            Ok(Some(user_session)) => Ok(Some(user_session)),
            Ok(None) => Ok(None),
            Err(err) => Err(AppError::internal(
                anyhow::Error::from(err).context("error while retrieving session"),
            )),
        }
    }
}

impl<S: Send + Sync> FromRequestParts<S> for UserSession {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        match <Self as OptionalFromRequestParts<S>>::from_request_parts(parts, state).await? {
            Some(user_session) => Ok(user_session),
            None => Err(AppError::unauthorized()),
        }
    }
}
