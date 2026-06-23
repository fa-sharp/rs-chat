use std::net::IpAddr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::UtcDateTime;

/// Active user session data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: Uuid,
}

impl UserSession {
    pub fn new(user_id: Uuid) -> Self {
        Self { user_id }
    }
}

/// Session metadata captured on login.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMeta {
    pub start_time: UtcDateTime,
    pub ip: Option<IpAddr>,
    pub user_agent: Option<String>,
}
