use std::time::Duration;

use anyhow::{Context, bail};
use axum_plugin::AdHocPlugin;
use tower_sessions::{CachingSessionStore, Expiry, SessionManagerLayer, cookie};
use tower_sessions_redis_store::RedisStore;

use crate::{
    db::DbPool,
    services::auth::{session::AuthSessionService, session_store::SessionDbStore},
    state::AppState,
};

const REDIS_PREFIX: &str = "rs-chat:sess:";
const CLEANUP_INTERVAL: Duration = Duration::from_mins(15);

/// Add session handling to the server. Sessions are stored in Postgres and cached in Redis.
pub fn plugin() -> AdHocPlugin<AppState> {
    AdHocPlugin::named("Session")
        .on_init(async |state| {
            let db_pool = state.get::<DbPool>().context("no db pool")?.clone();

            // Session cleanup task
            tokio::task::spawn(async move {
                let mut interval = tokio::time::interval(CLEANUP_INTERVAL);
                interval.tick().await;

                loop {
                    interval.tick().await;
                    tracing::debug!("Cleaning up auth sessions");
                    if let Err(err) = AuthSessionService::session_cleanup(&db_pool).await {
                        tracing::warn!("Error cleaning up auth sessions: {err}");
                    }
                }
            });

            Ok(state)
        })
        .on_setup(|router, state: &AppState| {
            let cookie_key = hex::decode(&state.config.auth.cookie_key)
                .context("cookie_key must be hex value")?;
            if cookie_key.len() < 32 {
                bail!("cookie_key must be at least 32 bytes");
            }

            // Session persistence
            let redis_store = RedisStore::with_prefix(state.redis.clone(), REDIS_PREFIX.to_owned());
            let db_store = SessionDbStore::new(state.db_pool.clone());
            let session_store = CachingSessionStore::new(redis_store, db_store);

            // Add session / cookie management to router
            let session_layer = SessionManagerLayer::new(session_store)
                .with_name(state.config.auth.cookie_name.clone())
                .with_expiry(Expiry::OnInactivity(cookie::time::Duration::minutes(15))) // default short session for login/OAuth
                .with_private(cookie::Key::derive_from(&cookie_key))
                .with_path("/")
                .with_secure(true)
                .with_http_only(true)
                .with_same_site(cookie::SameSite::Lax);

            Ok(router.layer(session_layer))
        })
}
