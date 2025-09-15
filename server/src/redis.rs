use std::{ops::Deref, time::Duration};

use fred::prelude::{Builder, Client, ClientLike, Pool, ReconnectPolicy, TcpConfig};
use rocket::{
    async_trait,
    fairing::AdHoc,
    request::{FromRequest, Outcome},
    Request,
};
use rocket_okapi::OpenApiFromRequest;

use crate::config::get_app_config;

/// Default size of the static Redis pool.
const REDIS_POOL_SIZE: usize = 4;
/// Timeout for connecting and executing commands.
const CLIENT_TIMEOUT: Duration = Duration::from_secs(6);

/// Fairing that sets up and initializes the Redis connection pool.
pub fn setup_redis() -> AdHoc {
    AdHoc::on_ignite("Redis", |rocket| async {
        let app_config = get_app_config(&rocket);
        let config = fred::prelude::Config::from_url(&app_config.redis_url)
            .expect("RS_CHAT_REDIS_URL should be valid Redis URL");

        // Build and initialize the Redis pool
        let pool = build_redis_pool(config, app_config.redis_pool.unwrap_or(REDIS_POOL_SIZE))
            .expect("Failed to build Redis pool");
        pool.init().await.expect("Redis connection failed");

        // Shutdown fairing
        let shutdown = AdHoc::on_shutdown("Shutdown Redis", |rocket| {
            Box::pin(async {
                if let Some(pool) = rocket.state::<Pool>() {
                    rocket::info!("Shutting down static Redis pool");
                    if let Err(err) = pool.quit().await {
                        rocket::warn!("Failed to shutdown Redis: {}", err);
                    }
                }
            })
        });

        rocket.manage(pool).attach(shutdown)
    })
}

fn build_redis_pool(
    redis_config: fred::prelude::Config,
    pool_size: usize,
) -> Result<Pool, fred::error::Error> {
    Builder::from_config(redis_config)
        .with_connection_config(|config| {
            config.connection_timeout = CLIENT_TIMEOUT;
            config.internal_command_timeout = CLIENT_TIMEOUT;
            config.max_command_attempts = 2;
            config.tcp = TcpConfig {
                nodelay: Some(true),
                ..Default::default()
            };
        })
        .set_policy(ReconnectPolicy::new_linear(0, 10_000, 1000))
        .with_performance_config(|config| {
            config.default_command_timeout = CLIENT_TIMEOUT;
        })
        .build_pool(pool_size)
}

/// Request guard for getting a Redis client from the connection pool.
#[derive(Debug, OpenApiFromRequest)]
pub struct RedisClient(Client);
impl Deref for RedisClient {
    type Target = Client;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[async_trait]
impl<'r> FromRequest<'r> for RedisClient {
    type Error = ();
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let pool = req.rocket().state::<Pool>().expect("exists");
        Outcome::Success(RedisClient(pool.next().clone()))
    }
}
