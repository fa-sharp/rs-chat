use std::time::Duration;

use chrono::Utc;
use rocket::fairing::AdHoc;
use rocket_flex_session::{storage::redis::RedisFredStorage, RocketFlexSession};
use uuid::Uuid;

use crate::config::get_app_config;

/// Type representing the session data.
#[derive(Debug, Clone)]
pub struct ChatRsAuthSession {
    pub user_id: Uuid,
    pub start_time: String,
}
impl ChatRsAuthSession {
    pub fn new(user_id: Uuid) -> Self {
        ChatRsAuthSession {
            user_id,
            start_time: Utc::now().to_rfc3339(),
        }
    }
}

/// Possible errors when parsing session data from Redis hash.
#[derive(thiserror::Error, Debug)]
pub enum SessionParseError {
    #[error("Missing field")]
    MissingField,
    #[error("Failed to parse")]
    ParsingError,
    #[error("Invalid field")]
    InvalidField,
}

/// Convert from Redis hash to session data.
impl TryFrom<fred::prelude::Value> for ChatRsAuthSession {
    type Error = SessionParseError;

    fn try_from(value: fred::prelude::Value) -> Result<Self, Self::Error> {
        let map = value
            .into_map()
            .map_err(|_| SessionParseError::ParsingError)?;
        let user_id = map
            .get(&"user_id".into())
            .and_then(|v| v.as_str())
            .ok_or(SessionParseError::MissingField)
            .and_then(|s| Uuid::try_parse(&s).map_err(|_| SessionParseError::InvalidField))?;
        let start_time = map
            .get(&"start_time".into())
            .and_then(|v| v.as_string())
            .ok_or(SessionParseError::MissingField)?;

        Ok(ChatRsAuthSession {
            user_id,
            start_time,
        })
    }
}

/// Convert from session data to Redis hash.
impl From<ChatRsAuthSession> for fred::prelude::Value {
    fn from(session: ChatRsAuthSession) -> Self {
        let mut hash = fred::types::Map::new();
        hash.insert("user_id".into(), session.user_id.to_string().into());
        hash.insert("start_time".into(), session.start_time.into());

        fred::prelude::Value::Map(hash)
    }
}

/// Fairing that sets up persistent sessions via Redis.
pub fn setup_session() -> AdHoc {
    AdHoc::on_ignite("Session setup", |rocket| async {
        let app_config = get_app_config(&rocket);
        let config = fred::prelude::Config::from_url(&app_config.redis_url)
            .expect("RS_CHAT_REDIS_URL should be valid Redis URL");
        let session_redis_pool = fred::prelude::Builder::from_config(config)
            .with_connection_config(|config| {
                config.connection_timeout = Duration::from_secs(4);
                config.tcp = fred::prelude::TcpConfig {
                    nodelay: Some(true),
                    ..Default::default()
                };
            })
            .build_pool(app_config.redis_pool.unwrap_or(2))
            .expect("Failed to build Redis session pool");
        let session_fairing: RocketFlexSession<ChatRsAuthSession> = RocketFlexSession::builder()
            .with_options(|opt| {
                opt.cookie_name = "auth_rs_chat".to_string();
                opt.ttl = Some(60 * 60 * 24 * 2); // 2 days
                opt.rolling = true;
            })
            .storage(RedisFredStorage::new(
                session_redis_pool,
                rocket_flex_session::storage::redis::RedisType::Hash,
                "sess:",
            ))
            .build();

        rocket.attach(session_fairing)
    })
}
