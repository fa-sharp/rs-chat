use std::{ops::Deref, time::Duration};

use fred::prelude::{Builder, Client, ClientLike, Config, Pool, TcpConfig};
use rocket::{
    fairing::AdHoc,
    http::Status,
    request::{FromRequest, Outcome},
};
use rocket_okapi::OpenApiFromRequest;

use crate::config::get_app_config;

/// Redis connection, available as a request guard. When used as a request parameter,
/// it will retrieve a connection from the managed Redis pool.
#[derive(OpenApiFromRequest)]
pub struct RedisClient(Client);
impl Deref for RedisClient {
    type Target = Client;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Retrieve a connection from the managed Redis pool. Responds with an
/// internal server error if Redis not initialized.
#[rocket::async_trait]
impl<'r> FromRequest<'r> for RedisClient {
    type Error = String;

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let Some(pool) = req.rocket().state::<Pool>() else {
            return Outcome::Error((
                Status::InternalServerError,
                "Redis not initialized".to_owned(),
            ));
        };
        Outcome::Success(RedisClient(pool.next().clone()))
    }
}

/// Fairing that sets up and initializes the Redis database
pub fn setup_redis() -> AdHoc {
    AdHoc::on_ignite("Redis", |rocket| async {
        rocket
            .attach(AdHoc::on_ignite(
                "Initialize Redis connection",
                |rocket| async {
                    let app_config = get_app_config(&rocket);
                    let config = Config::from_url(&app_config.redis_url)
                        .expect("RS_CHAT_REDIS_URL should be valid Redis URL");
                    let pool = Builder::from_config(config)
                        .with_connection_config(|config| {
                            config.connection_timeout = Duration::from_secs(4);
                            config.tcp = TcpConfig {
                                nodelay: Some(true),
                                ..Default::default()
                            };
                        })
                        .build_pool(app_config.redis_pool.unwrap_or(4))
                        .expect("Failed to build Redis pool");
                    pool.init().await.expect("Failed to connect to Redis");

                    rocket.manage(pool)
                },
            ))
            .attach(AdHoc::on_shutdown("Shutdown Redis connection", |rocket| {
                Box::pin(async {
                    if let Some(pool) = rocket.state::<Pool>() {
                        rocket::info!("Shutting down Redis connection");
                        if let Err(err) = pool.quit().await {
                            rocket::error!("Failed to shutdown Redis: {}", err);
                        }
                    }
                })
            }))
    })
}
