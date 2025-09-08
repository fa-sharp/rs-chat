use chrono::Utc;
use fred::{
    prelude::{Error as FredError, ErrorKind, FredResult, Key, Pool, Value},
    types::{FromValue, Map},
};
use rocket::fairing::AdHoc;
use rocket_flex_session::{
    storage::redis::{RedisFredStorage, RedisFredStorageIndexed, RedisType},
    RocketFlexSession, SessionIdentifier,
};
use uuid::Uuid;

/// Keys used in the session data.
mod keys {
    use fred::types::Key;
    pub const USER_ID_KEY: Key = Key::from_static_str("user_id");
    pub const USER_ID_BYTES_KEY: Key = Key::from_static_str("user_id_bytes");
    pub const START_TIME_KEY: Key = Key::from_static_str("start_time");
}

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
impl FromValue for ChatRsAuthSession {
    fn from_value(value: Value) -> Result<Self, FredError> {
        let map = value.into_map()?;
        let user_id = uuid_from_map(&map, &keys::USER_ID_BYTES_KEY, "Invalid/missing user ID")?;
        let start_time = string_from_map(&map, &keys::START_TIME_KEY, "Missing start time")?;
        Ok(ChatRsAuthSession {
            user_id,
            start_time,
        })
    }
}
fn string_from_map(map: &Map, key: &Key, err: &'static str) -> FredResult<String> {
    let string = map.get(&key).and_then(Value::as_string);
    string.ok_or(FredError::new(ErrorKind::Parse, err))
}
fn uuid_from_map(map: &Map, key: &Key, err: &'static str) -> FredResult<Uuid> {
    let bytes = map.get(key).and_then(Value::as_bytes);
    let uuid = bytes.and_then(|bytes| Uuid::from_slice(bytes).ok());
    uuid.ok_or(FredError::new(ErrorKind::Parse, err))
}

/// Convert from session data to Redis hash.
impl TryFrom<ChatRsAuthSession> for Value {
    type Error = FredError;
    fn try_from(session: ChatRsAuthSession) -> Result<Self, Self::Error> {
        let user_id_bytes = session.user_id.as_bytes().as_slice();
        let map = Map::try_from([
            (keys::USER_ID_KEY, Value::from(session.user_id.to_string())),
            (keys::USER_ID_BYTES_KEY, Value::from(user_id_bytes)),
            (keys::START_TIME_KEY, Value::from(session.start_time)),
        ])?;
        Ok(Value::Map(map))
    }
}

/// Rocket fairing that sets up persistent sessions via Redis.
pub fn setup_session() -> AdHoc {
    AdHoc::on_ignite("Sessions", |rocket| async {
        let pool = rocket.state::<Pool>().expect("pool exists");
        let storage = RedisFredStorage::builder()
            .pool(pool.clone())
            .prefix("sess:")
            .redis_type(RedisType::Hash)
            .build();
        let session_fairing = RocketFlexSession::<ChatRsAuthSession>::builder()
            .with_options(|opt| {
                opt.cookie_name = "auth_rs_chat".to_string();
                opt.ttl = Some(60 * 60 * 24 * 2); // 2 days
                opt.rolling = true;
            })
            .storage(RedisFredStorageIndexed::from_storage(storage).build())
            .build();

        rocket.attach(session_fairing)
    })
}
