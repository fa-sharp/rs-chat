use std::{ops::Deref, sync::Arc, time::Duration};

use deadpool::managed;
use fred::prelude::{Builder, Client, ClientLike, ReconnectPolicy, TcpConfig};
use rocket::{
    async_trait,
    fairing::AdHoc,
    http::Status,
    outcome::try_outcome,
    request::{FromRequest, Outcome},
    Request, State,
};
use rocket_okapi::OpenApiFromRequest;
use tokio::sync::Mutex;

use crate::config::get_app_config;

/// Default size of the static Redis pool.
const REDIS_POOL_SIZE: usize = 4;
/// Default maximum number of concurrent exclusive clients (e.g. max concurrent streams)
const MAX_EXCLUSIVE_CLIENTS: usize = 20;
/// Timeout for connecting and executing commands.
const CLIENT_TIMEOUT: Duration = Duration::from_secs(6);
/// Interval for checking idle exclusive clients.
const IDLE_TASK_INTERVAL: Duration = Duration::from_secs(30);
/// Shut down exclusive clients after this period of inactivity.
const IDLE_TIME: Duration = Duration::from_secs(60);

/// Fairing that sets up and initializes the Redis connection pool.
pub fn setup_redis() -> AdHoc {
    AdHoc::on_ignite("Redis", |rocket| async {
        rocket
            .attach(AdHoc::on_ignite(
                "Initialize Redis connection",
                |rocket| async {
                    let app_config = get_app_config(&rocket);
                    let config = fred::prelude::Config::from_url(&app_config.redis_url)
                        .expect("RS_CHAT_REDIS_URL should be valid Redis URL");

                    // Build and initialize the static Redis pool
                    let static_pool =
                        build_redis_pool(config, app_config.redis_pool.unwrap_or(REDIS_POOL_SIZE))
                            .expect("Failed to build static Redis pool");
                    static_pool.init().await.expect("Redis connection failed");

                    // Build and initialize the dynamic, exclusive Redis pool for long-running tasks
                    let exclusive_manager = ExclusiveClientManager::new(static_pool.clone());
                    let exclusive_pool: ExclusiveClientPool =
                        managed::Pool::builder(exclusive_manager)
                            .max_size(app_config.max_streams.unwrap_or(MAX_EXCLUSIVE_CLIENTS))
                            .runtime(deadpool::Runtime::Tokio1)
                            .create_timeout(Some(CLIENT_TIMEOUT))
                            .recycle_timeout(Some(CLIENT_TIMEOUT))
                            .wait_timeout(Some(CLIENT_TIMEOUT))
                            .build()
                            .expect("Failed to build exclusive Redis pool");

                    // Spawn a task to periodically clean up idle exclusive clients
                    let idle_task_pool = exclusive_pool.clone();
                    tokio::spawn(async move {
                        let mut interval = tokio::time::interval(IDLE_TASK_INTERVAL);
                        loop {
                            interval.tick().await;
                            idle_task_pool.retain(|_, metrics| metrics.last_used() < IDLE_TIME);
                        }
                    });

                    rocket.manage(static_pool).manage(exclusive_pool)
                },
            ))
            .attach(AdHoc::on_shutdown("Shutdown Redis connection", |rocket| {
                Box::pin(async {
                    if let Some(pool) = rocket.state::<fred::clients::Pool>() {
                        rocket::info!("Shutting down static Redis pool");
                        if let Err(err) = pool.quit().await {
                            rocket::warn!("Failed to shutdown Redis: {}", err);
                        }
                    }
                    if let Some(exclusive_pool) = rocket.state::<ExclusiveClientPool>() {
                        rocket::info!("Shutting down exclusive Redis pool");
                        for client in exclusive_pool.manager().clients.lock().await.iter() {
                            if let Err(err) = client.quit().await {
                                rocket::warn!("Failed to shutdown Redis client: {}", err);
                            }
                        }
                    }
                })
            }))
    })
}

pub fn build_redis_pool(
    redis_config: fred::prelude::Config,
    pool_size: usize,
) -> Result<fred::clients::Pool, fred::error::Error> {
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

/// A pool of exclusive Redis connections for long-running tasks.
pub type ExclusiveClientPool = managed::Pool<ExclusiveClientManager>;

/// Deadpool implementation for a pool of exclusive Redis clients.
#[derive(Debug)]
pub struct ExclusiveClientManager {
    pool: fred::clients::Pool,
    clients: Arc<Mutex<Vec<Client>>>,
}
impl ExclusiveClientManager {
    pub fn new(pool: fred::clients::Pool) -> Self {
        Self {
            pool,
            clients: Arc::default(),
        }
    }
}
impl managed::Manager for ExclusiveClientManager {
    type Type = Client;
    type Error = fred::error::Error;

    async fn create(&self) -> Result<Client, Self::Error> {
        let client = self.pool.next().clone_new();
        client.init().await?;
        self.clients.lock().await.push(client.clone());
        Ok(client)
    }
    async fn recycle(
        &self,
        client: &mut Client,
        _: &managed::Metrics,
    ) -> managed::RecycleResult<Self::Error> {
        if !client.is_connected() {
            client.init().await?;
        }
        let _: () = client.ping(None).await?;
        Ok(())
    }
    fn detach(&self, client: &mut Self::Type) {
        let client = client.clone();
        let clients = self.clients.clone();
        tokio::spawn(async move {
            clients.lock().await.retain(|c| c.id() != client.id());
            if let Err(err) = client.quit().await {
                rocket::error!("Failed to disconnect Redis client: {}", err);
            }
        });
    }
}

/// Request guard to get an exclusive Redis connection for long-running operations.
#[derive(Debug, OpenApiFromRequest)]
pub struct ExclusiveRedisClient(pub managed::Object<ExclusiveClientManager>);

impl Deref for ExclusiveRedisClient {
    type Target = Client;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
impl<'r> FromRequest<'r> for ExclusiveRedisClient {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let pool = try_outcome!(req.guard::<&State<ExclusiveClientPool>>().await);
        match pool.get().await {
            Ok(client) => Outcome::Success(ExclusiveRedisClient(client)),
            Err(err) => {
                rocket::error!("Failed to initialize Redis client: {}", err);
                Outcome::Error((Status::InternalServerError, ()))
            }
        }
    }
}
