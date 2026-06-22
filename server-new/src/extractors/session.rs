use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use anyhow::Context;
use axum::{
    extract::{ConnectInfo, FromRequestParts, OptionalFromRequestParts},
    http::header,
};
use serde::{Deserialize, Serialize};
use time::UtcDateTime;
use tower_sessions::Session;
use uuid::Uuid;

use crate::{error::AppError, state::AppState};

/// The field used to store the user session data
const USER_SESSION_FIELD: &str = "sess";
/// The field used to store the user session metadata
const SESSION_META_FIELD: &str = "meta";

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
    pub user_id: Uuid,
}

/// Session metadata extracted on login.
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionMeta {
    pub start_time: UtcDateTime,
    pub ip: Option<IpAddr>,
    pub user_agent: Option<String>,
}

impl UserSession {
    pub fn new(user_id: Uuid) -> Self {
        Self { user_id }
    }

    pub async fn init(
        session: &Session,
        meta: &SessionMeta,
        user_id: &Uuid,
    ) -> Result<(), AppError> {
        session
            .insert(USER_SESSION_FIELD, UserSession::new(user_id.clone()))
            .await
            .context("failed to initialize session")?;
        session.insert(SESSION_META_FIELD, meta).await.ok();

        Ok(())
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

impl FromRequestParts<AppState> for SessionMeta {
    type Rejection = ();

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let ip_header = state
            .config
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
        let start_time = UtcDateTime::now();

        Ok(Self {
            start_time,
            ip,
            user_agent,
        })
    }
}
