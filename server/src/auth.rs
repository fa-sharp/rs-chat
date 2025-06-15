use std::time::Duration;

use rocket::{
    fairing::AdHoc,
    http::Status,
    request::{FromRequest, Outcome},
};
use rocket_flex_session::{
    storage::{
        memory::MemoryStorage,
        redis::{RedisFredStorage, RedisType},
    },
    RocketFlexSession, Session,
};
use rocket_oauth2::{HyperRustlsAdapter, OAuthConfig, StaticProvider};
use uuid::Uuid;

use crate::{
    api::GitHubUserInfo,
    config::get_app_config,
    db::{models::ChatRsUser, services::user::UserDbService, DbConnection},
};

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

/// Session request guard / middleware to ensure a logged-in user.
#[rocket::async_trait]
impl<'r> FromRequest<'r> for ChatRsUser {
    type Error = &'r str;

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let session = req
            .guard::<Session<ChatRsAuthSession>>()
            .await
            .expect("should not fail");

        let Some(user_id) = session.tap(|session| match session {
            Some(data) => Some(data.user_id),
            None => None,
        }) else {
            return Outcome::Error((Status::Unauthorized, "Unauthorized"));
        };

        let Outcome::Success(mut db) = req.guard::<DbConnection>().await else {
            rocket::error!("Session guard: database connection failed");
            return Outcome::Error((Status::InternalServerError, "Server error"));
        };
        let user = UserDbService::new(&mut db).find_by_id(&user_id).await;
        match user {
            Ok(Some(user)) => Outcome::Success(user),
            Ok(None) => Outcome::Error((Status::NotFound, "User not found")),
            Err(e) => {
                rocket::error!("Session guard: database error: {}", e);
                Outcome::Error((Status::InternalServerError, "Server error"))
            }
        }
    }
}

/// Fairing that sets up OAuth login
pub fn setup_oauth() -> AdHoc {
    AdHoc::on_ignite("OAuth setup", |rocket| async {
        let app_config = get_app_config(&rocket);
        let oauth_config = OAuthConfig::new(
            StaticProvider::GitHub,
            app_config.github_client_id.to_owned(),
            app_config.github_client_secret.to_owned(),
            Some(format!(
                "{}/api/oauth/login/github/callback",
                app_config.server_address
            )),
        );
        rocket.attach(rocket_oauth2::OAuth2::<GitHubUserInfo>::custom(
            HyperRustlsAdapter::default(),
            oauth_config,
        ))
    })
}

/// Fairing that sets up persistent sessions in Rocket. Uses
/// Redis if CHAT_RS_REDIS_URL is set, otherwise uses in-memory session store.
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
                    .with_options(|opt| {
                        opt.cookie_name = "auth_rs_chat".to_string();
                        opt.ttl = Some(60 * 60 * 24 * 2); // 2 days
                        opt.rolling = true;
                    })
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
