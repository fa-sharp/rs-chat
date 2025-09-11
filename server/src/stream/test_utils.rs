#[cfg(test)]
pub(super) async fn setup_redis_pool() -> crate::redis::ExclusiveClientPool {
    use crate::redis::{ExclusiveClientManager, ExclusiveClientPool};
    use fred::prelude::{Builder, ClientLike, Config};

    let url = std::env::var("RS_CHAT_REDIS_URL").unwrap_or("redis://127.0.0.1".to_owned());
    let config = Config::from_url(&url).unwrap();
    let pool = Builder::from_config(config)
        .build_pool(1)
        .expect("Failed to build Redis pool");
    pool.init().await.expect("Failed to connect to Redis");

    let manager = ExclusiveClientManager::new(pool.clone());
    let deadpool: ExclusiveClientPool = deadpool::managed::Pool::builder(manager)
        .max_size(3)
        .build()
        .unwrap();

    deadpool
}
