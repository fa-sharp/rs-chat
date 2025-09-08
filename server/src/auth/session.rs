use chrono::Utc;
use fred::types::Key;
use rocket::fairing::AdHoc;
use rocket_flex_session::{
    storage::redis::{RedisFredStorage, RedisFredStorageIndexed},
    RocketFlexSession, SessionIdentifier,
};
use uuid::Uuid;

const USER_ID_KEY: Key = Key::from_static_str("user_id");
const USER_ID_BYTES_KEY: Key = Key::from_static_str("user_id_bytes");
const START_TIME_KEY: Key = Key::from_static_str("start_time");

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
impl SessionIdentifier for ChatRsAuthSession {
    const IDENTIFIER: &str = "user";
    type Id = Uuid;

    fn identifier(&self) -> Option<&Self::Id> {
        Some(&self.user_id)
    }
}

/// Convert from Redis hash to session data.
impl fred::prelude::FromValue for ChatRsAuthSession {
    fn from_value(value: fred::prelude::Value) -> Result<Self, fred::prelude::Error> {
        use fred::prelude::{Error, ErrorKind, Value};
        let map = value.into_map()?;
        let user_id = map
            .get(&USER_ID_BYTES_KEY)
            .and_then(Value::as_bytes)
            .and_then(|bytes| Uuid::from_slice(bytes).ok())
            .ok_or(Error::new(ErrorKind::Parse, "Invalid/missing user ID"))?;
        let start_time = map
            .get(&START_TIME_KEY)
            .and_then(Value::as_string)
            .unwrap_or_default();
        Ok(ChatRsAuthSession {
            user_id,
            start_time,
        })
    }
}

/// Convert from session data to Redis hash.
impl TryFrom<ChatRsAuthSession> for fred::prelude::Value {
    type Error = fred::error::Error;

    fn try_from(session: ChatRsAuthSession) -> Result<Self, Self::Error> {
        use fred::types::{Map, Value};
        let user_id_bytes = session.user_id.as_bytes().as_slice();
        let map = Map::try_from([
            (USER_ID_KEY, Value::from(session.user_id.to_string())),
            (USER_ID_BYTES_KEY, Value::from(user_id_bytes)),
            (START_TIME_KEY, Value::from(session.start_time)),
        ])?;
        Ok(Value::Map(map))
    }
}

/// Fairing that sets up persistent sessions via Redis.
pub fn setup_session() -> AdHoc {
    AdHoc::on_ignite("Sessions", |rocket| async {
        let pool = rocket.state::<fred::clients::Pool>().expect("pool exists");
        let storage = RedisFredStorage::new(
            pool.clone(),
            rocket_flex_session::storage::redis::RedisType::Hash,
            "sess:",
        );
        let session_fairing = RocketFlexSession::<ChatRsAuthSession>::builder()
            .with_options(|opt| {
                opt.cookie_name = "auth_rs_chat".to_string();
                opt.ttl = Some(60 * 60 * 24 * 2); // 2 days
                opt.rolling = true;
            })
            .storage(RedisFredStorageIndexed::new(storage, None))
            .build();

        rocket.attach(session_fairing)
    })
}
