use std::time::Duration;

use rocket::fairing::AdHoc;
use rocket_flex_session::{
    storage::{
        memory::MemoryStorage,
        redis::{RedisFredStorage, RedisType},
    },
    RocketFlexSession,
};
use uuid::Uuid;

use crate::config::get_app_config;

#[derive(Debug, Clone)]
pub struct ChatRsAuthSession {
    pub user_id: Uuid,
}
impl ChatRsAuthSession {
    pub fn new(user_id: Uuid) -> Self {
        ChatRsAuthSession { user_id }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SessionParseError {
    #[error("Missing field")]
    MissingField,
    #[error("Failed to parse")]
    ParsingError,
    #[error("Invalid field")]
    InvalidField,
}

impl TryFrom<fred::prelude::Value> for ChatRsAuthSession {
    type Error = SessionParseError;

    fn try_from(value: fred::prelude::Value) -> Result<Self, Self::Error> {
        let map = value
            .into_map()
            .map_err(|_| SessionParseError::ParsingError)?;
        let user_id = map
            .get(&fred::types::Key::from_static_str("user_id"))
            .ok_or(SessionParseError::MissingField)?
            .as_bytes()
            .ok_or(SessionParseError::ParsingError)?;
        let user_id =
            Uuid::try_parse_ascii(&user_id).map_err(|_| SessionParseError::InvalidField)?;

        Ok(ChatRsAuthSession { user_id })
    }
}

impl From<ChatRsAuthSession> for fred::prelude::Value {
    fn from(session: ChatRsAuthSession) -> Self {
        let mut hash = fred::types::Map::new();
        hash.insert("user_id".into(), session.user_id.to_string().into());
        fred::prelude::Value::Map(hash)
    }
}

/// Fairing that sets up persistent sessions
pub fn setup_session() -> AdHoc {
    AdHoc::on_ignite("Session setup", |rocket| async {
        let app_config = get_app_config(&rocket);

        if let Some(redis_url) = Some(&app_config.redis_url) {
            let config = fred::prelude::Config::from_url(&redis_url)
                .expect("CHAT_RS_REDIS_URL should be valid Redis URL");
            let session_redis_pool = fred::prelude::Builder::from_config(config)
                .with_connection_config(|config| {
                    config.connection_timeout = Duration::from_secs(4);
                    config.tcp = fred::prelude::TcpConfig {
                        nodelay: Some(true),
                        ..Default::default()
                    };
                })
                .build_pool(2)
                .expect("Failed to build Redis session pool");
            let session_fairing: RocketFlexSession<ChatRsAuthSession> =
                RocketFlexSession::builder()
                    .storage(RedisFredStorage::new(
                        session_redis_pool,
                        RedisType::Hash,
                        "sess:",
                    ))
                    .build();

            rocket.attach(session_fairing)
        } else {
            let session_fairing: RocketFlexSession<ChatRsAuthSession> =
                RocketFlexSession::builder()
                    .storage(MemoryStorage::default())
                    .build();

            rocket.attach(session_fairing)
        }
    })
}
