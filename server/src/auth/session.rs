use std::{ops::Deref, time::Duration};

use chrono::Utc;
use rocket::fairing::AdHoc;
use rocket_flex_session::{storage::redis::RedisFredStorage, RocketFlexSession};
use uuid::Uuid;

use crate::config::get_app_config;

const USER_ID_KEY: &str = "user_id";
const USER_ID_BYTES_KEY: &str = "user_id_bytes";
const START_TIME_KEY: &str = "start_time";

/// Type representing the session data.
#[derive(Debug, Clone)]
pub struct ChatRsAuthSession(fred::types::Map);

impl Deref for ChatRsAuthSession {
    type Target = fred::types::Map;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ChatRsAuthSession {
    pub fn new(user_id: Uuid) -> Self {
        let mut hash = fred::types::Map::new();
        hash.insert(USER_ID_KEY.into(), user_id.to_string().into());
        hash.insert(
            USER_ID_BYTES_KEY.into(),
            user_id.as_bytes().as_slice().into(),
        );
        hash.insert(START_TIME_KEY.into(), Utc::now().to_rfc3339().into());
        ChatRsAuthSession(hash)
    }

    pub fn user_id(&self) -> Option<Uuid> {
        self.get(&fred::types::Key::from_static_str(USER_ID_BYTES_KEY))
            .and_then(|val| val.as_bytes())
            .and_then(|bytes| Uuid::from_slice(bytes).ok())
    }
}

/// Possible errors when parsing session data from Redis hash.
#[derive(thiserror::Error, Debug)]
pub enum SessionParseError {
    #[error("Failed to parse")]
    ParsingError,
}

/// Convert from Redis hash to session data.
impl TryFrom<fred::prelude::Value> for ChatRsAuthSession {
    type Error = SessionParseError;

    fn try_from(value: fred::prelude::Value) -> Result<Self, Self::Error> {
        let map = value
            .into_map()
            .map_err(|_| SessionParseError::ParsingError)?;
        Ok(ChatRsAuthSession(map))
    }
}

/// Convert from session data to Redis hash.
impl From<ChatRsAuthSession> for fred::prelude::Value {
    fn from(session: ChatRsAuthSession) -> Self {
        fred::types::Value::Map(session.0)
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
