use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use axum::{
    extract::{ConnectInfo, FromRequestParts, OptionalFromRequestParts},
    http::header,
};
use tower_sessions::Session;

use crate::{error::AppError, state::AppState};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::UtcDateTime;

/// Represents an active user session. This can be used as an extractor in route handlers:
/// - If used as `UserSession`, request will automatically return an unauthorized error
/// if there is no active session.
/// - If used as `Option<UserSession>`, will be `Some` if there is an active session
/// and `None` otherwise.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: Uuid,
}

impl UserSession {
    pub fn new(user_id: Uuid) -> Self {
        Self { user_id }
    }
}

/// Session metadata extracted on login.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMeta {
    pub start_time: UtcDateTime,
    pub ip: Option<IpAddr>,
    pub user_agent: Option<String>,
}

/// Active user session data.
impl OptionalFromRequestParts<AppState> for UserSession {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Option<Self>, Self::Rejection> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|(_, msg)| AppError::internal(anyhow::anyhow!(msg)))?;
        let user_session = state
            .auth_service()
            .session()
            .user_session(&session)
            .await?;

        Ok(user_session)
    }
}

impl FromRequestParts<AppState> for UserSession {
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

impl FromRequestParts<AppState> for SessionMeta {
    type Rejection = ();

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let ip_header = state
            .config
            .server
            .ip_header
            .as_ref()
            .and_then(|h| parts.headers.get(h).and_then(|h| h.to_str().ok()))
            .and_then(|h| IpAddr::from_str(h).ok());
        let ip = match ip_header {
            Some(ip) => Some(ip),
            None => ConnectInfo::<SocketAddr>::from_request_parts(parts, state)
                .await
                .ok()
                .map(|info| info.ip()),
        };
        let user_agent = parts
            .headers
            .get(header::USER_AGENT)
            .and_then(|h| h.to_str().ok())
            .map(|ua| ua.to_owned());

        Ok(Self {
            ip,
            user_agent,
            start_time: chrono::Utc::now(),
        })
    }
}
