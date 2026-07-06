use aide::OperationIo;
use anyhow::anyhow;
use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts},
    http::header,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{db::DbService, error::AppError, state::AppState};

/**
Represents an active user, extracted from the session, proxy headers, or API key. This can be used
as an extractor in route handlers:
- If used as `CurrentUser`, request will automatically return an unauthorized error
if there is no active user.
- If used as `Option<CurrentUser>`, will be `Some` if there is an active user
and `None` otherwise.
*/
#[derive(Debug, Clone, Serialize, Deserialize, OperationIo)]
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
        let auth_service = state.auth_service();

        // If there is an Authorization header, validate the API key
        if let Some(auth_header) = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|bytes| bytes.to_str().ok())
        {
            let user_id = auth_service
                .api_keys()
                .validate_api_key(&state.db_pool, auth_header)
                .await?;

            Ok(Some(Self { user_id }))
        } else {
            // If SSO header / proxy auth is enabled, check forwarded headers first
            if state.config.auth.proxy.enabled {
                let proxy_service = auth_service.proxy();
                if let Some(proxy_user) = proxy_service.extract_proxy_user(&parts.headers)? {
                    let mut db = DbService::from_pool(&state.db_pool).await?;
                    match proxy_service.find_proxy_user(&mut db, &proxy_user).await? {
                        Some(user_id) => return Ok(Some(Self::new(user_id))),
                        None => {
                            let new_user = proxy_service
                                .create_proxy_user(&mut db, &proxy_user)
                                .await?;
                            return Ok(Some(Self::new(new_user.id)));
                        }
                    }
                }
            }

            // Check for session
            let session = parts
                .extensions
                .get::<tower_sessions::Session>()
                .ok_or_else(|| AppError::internal(anyhow!("session not attached to request")))?;
            let maybe_user_id = auth_service.session().active_user_id(&session).await?;

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
            Some(current_user) => Ok(current_user),
            None => Err(AppError::unauthorized("no active user")),
        }
    }
}
