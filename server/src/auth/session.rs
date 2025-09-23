use chrono::Utc;
use rocket::fairing::AdHoc;
use rocket_flex_session::{
    error::SessionError,
    storage::redis::{RedisFormat, RedisFredStorage, RedisValue, SessionRedis},
    RocketFlexSession, SessionIdentifier,
};
use uuid::Uuid;

use crate::auth::session_meta::SessionMeta;

/// Type representing the session data.
#[derive(Debug, Clone)]
pub struct ChatRsAuthSession {
    pub user_id: Uuid,
    pub start_time: Option<String>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
}

/// Rocket fairing that sets up persistent sessions via Redis.
pub fn setup_session() -> AdHoc {
    AdHoc::on_ignite("Sessions", |rocket| async {
        let pool = rocket.state::<fred::prelude::Pool>().expect("pool exists");
        let storage = RedisFredStorage::builder()
            .pool(pool.clone())
            .prefix("sess:")
            .index_prefix("sess:user:")
            .build();
        let session_fairing = RocketFlexSession::<ChatRsAuthSession>::builder()
            .with_options(|opt| {
                opt.cookie_name = "auth_rs_chat".to_string();
                opt.ttl = Some(60 * 60 * 24 * 2); // 2 days
                opt.rolling = true;
            })
            .storage(storage)
            .build();

        rocket.attach(session_fairing)
    })
}

impl ChatRsAuthSession {
    pub fn new(user_id: Uuid, meta: SessionMeta) -> Self {
        ChatRsAuthSession {
            user_id,
            start_time: Some(Utc::now().to_rfc3339()),
            ip: meta.ip.map(|ip| ip.to_string()),
            user_agent: meta.user_agent.map(|ua| ua.to_owned()),
        }
    }
}
impl SessionIdentifier for ChatRsAuthSession {
    type Id = String;

    /// Group sessions by user ID, using lowercase hex keys to track each user's sessions.
    fn identifier(&self) -> Option<Self::Id> {
        Some(hex::encode(self.user_id.as_bytes()))
    }
}

/// Keys used in the session data.
mod keys {
    pub const USER_ID_HEX_KEY: &str = "user_id";
    pub const START_TIME_KEY: &str = "start";
    pub const IP_KEY: &str = "ip";
    pub const USER_AGENT_KEY: &str = "ua";
}

impl SessionRedis for ChatRsAuthSession {
    const REDIS_FORMAT: RedisFormat = RedisFormat::Map;
    type Error = SessionError;

    fn into_redis(self) -> Result<RedisValue, Self::Error> {
        let user_id_bytes = self.user_id.as_bytes();
        let mut data_pairs = vec![(keys::USER_ID_HEX_KEY.into(), hex::encode(user_id_bytes))];
        for (key, optional_val) in [
            (keys::START_TIME_KEY.into(), self.start_time),
            (keys::IP_KEY.into(), self.ip),
            (keys::USER_AGENT_KEY.into(), self.user_agent),
        ] {
            if let Some(val) = optional_val {
                data_pairs.push((key, val));
            }
        }

        Ok(RedisValue::Map(data_pairs))
    }

    fn from_redis(value: RedisValue) -> Result<Self, Self::Error> {
        let map = value.into_map().expect("should be a map");
        let (mut user_id, mut start_time, mut ip, mut user_agent) = (None, None, None, None);
        for (key, val) in map {
            match key.as_str() {
                keys::USER_ID_HEX_KEY => {
                    let mut bytes = [0_u8; 16];
                    hex::decode_to_slice(val, &mut bytes)
                        .map_err(|e| SessionError::Parsing(e.into()))?;
                    user_id = Some(Uuid::from_bytes(bytes))
                }
                keys::START_TIME_KEY => start_time = Some(val),
                keys::IP_KEY => ip = Some(val),
                keys::USER_AGENT_KEY => user_agent = Some(val),
                _ => (),
            }
        }

        Ok(Self {
            user_id: user_id.ok_or(SessionError::InvalidData)?,
            start_time,
            ip,
            user_agent,
        })
    }
}
