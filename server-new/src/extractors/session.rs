use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use aide::OperationIo;
use anyhow::anyhow;
use axum::{
    extract::{ConnectInfo, FromRequestParts},
    http::header,
};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

use crate::{db::UtcDateTime, error::AppError, state::AppState};

/// Extractor to get raw session and request metadata
#[derive(OperationIo)]
pub struct AppSession {
    pub session: Session,
    pub meta: SessionMeta,
}

/// Session metadata extracted on login.
#[derive(Debug, Clone, Serialize, Deserialize, OperationIo)]
pub struct SessionMeta {
    pub start_time: UtcDateTime,
    pub ip: Option<IpAddr>,
    pub user_agent: Option<String>,
}

impl FromRequestParts<AppState> for AppSession {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let meta = SessionMeta::from_request_parts(parts, state).await?;
        let session = parts
            .extensions
            .get::<Session>()
            .cloned()
            .ok_or_else(|| AppError::internal(anyhow!("session not attached to request")))?;

        Ok(Self { session, meta })
    }
}

impl FromRequestParts<AppState> for SessionMeta {
    type Rejection = AppError;

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
