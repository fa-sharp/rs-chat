use anyhow::anyhow;
use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts},
    http::header,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{db::DbService, error::AppError, state::AppState};

/**
Represents an active user, extracted from the session or API key. This can be used
as an extractor in route handlers:
- If used as `CurrentUser`, request will automatically return an unauthorized error
if there is no active user.
- If used as `Option<CurrentUser>`, will be `Some` if there is an active user
and `None` otherwise.
*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentUser {
    pub user_id: Uuid,
}

impl CurrentUser {
    fn new(user_id: Uuid) -> Self {
        Self { user_id }
    }
}

impl OptionalFromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Option<Self>, Self::Rejection> {
        // If there is an Authorization header, validate the API key
        if let Some(auth_header) = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|bytes| bytes.to_str().ok())
        {
            let mut db = DbService::from_pool(&state.db_pool)
                .await
                .map_err(|err| AppError::internal(err.into()))?;
            let user_id = state
                .auth_service()
                .api_keys()
                .validate_api_key(&mut db, auth_header)
                .await?;

            Ok(Some(Self { user_id }))
        }
        // Check for session
        else {
            let session = parts
                .extensions
                .get::<tower_sessions::Session>()
                .ok_or_else(|| AppError::internal(anyhow!("session not attached to request")))?;
            let maybe_user_id = state
                .auth_service()
                .session()
                .active_user_id(&session)
                .await?;

            Ok(maybe_user_id.map(Self::new))
        }
    }
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match <Self as OptionalFromRequestParts<AppState>>::from_request_parts(parts, state).await?
        {
            Some(user_session) => Ok(user_session),
            None => Err(AppError::unauthorized("no active session")),
        }
    }
}
