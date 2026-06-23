use anyhow::{Context, bail};
use axum_plugin::AdHocPlugin;
use tower_sessions::{
    CachingSessionStore, Expiry, SessionManagerLayer,
    cookie::{Key, SameSite, time::Duration},
};
use tower_sessions_redis_store::RedisStore;

use crate::{services::SessionDbStore, state::AppState};

const REDIS_PREFIX: &str = "rs-chat:sess:";

/// Add session handling to the server. Sessions are stored in Postgres and cached in Redis.
pub fn plugin() -> AdHocPlugin<AppState> {
    AdHocPlugin::named("Session").on_setup(|router, state: &AppState| {
        let cookie_key =
            hex::decode(&state.config.auth.cookie_key).context("cookie_key must be hex value")?;
        if cookie_key.len() < 32 {
            bail!("cookie_key must be at least 32 bytes");
        }

        let redis_store = RedisStore::with_prefix(state.redis.clone(), REDIS_PREFIX.to_owned());
        let db_store = SessionDbStore::new(state.db_pool.clone());
        let session_store = CachingSessionStore::new(redis_store, db_store);
        let session_layer = SessionManagerLayer::new(session_store)
            .with_name(state.config.auth.cookie_name.clone())
            .with_expiry(Expiry::OnInactivity(Duration::seconds(
                state.config.auth.session_length,
            )))
            .with_private(Key::derive_from(&cookie_key))
            .with_path("/")
            .with_secure(true)
            .with_http_only(true)
            .with_same_site(SameSite::Lax);

        Ok(router.layer(session_layer))
    })
}
