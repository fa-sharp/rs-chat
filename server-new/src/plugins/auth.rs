use std::time::Duration;

use anyhow::{Context, bail};
use tower_sessions::{CachingSessionStore, Expiry, SessionManagerLayer, cookie};
use tower_sessions_redis_store::RedisStore;

use crate::{
    db::DbPool,
    plugins::AxumPlugin,
    services::auth::{
        encryption::Encryptor, oauth::OAuthService, session::AuthSessionService,
        session_store::SessionDbStore,
    },
};

const REDIS_PREFIX: &str = "rs-chat:sess:";
const CLEANUP_INTERVAL: Duration = Duration::from_mins(15);

/// Add auth & session handling to the server. Sessions are stored in Postgres and cached in Redis.
pub fn plugin() -> AxumPlugin {
    AxumPlugin::named("Auth")
        .on_init(async |mut app| {
            // Verify encryption key and build encryptor
            let encryption_key = hex::decode(&app.config().auth.encryption_key)
                .context("encryption_key must be hex value")?;
            if encryption_key.len() != 32 {
                bail!("encryption_key must be 32 bytes");
            }
            let encryptor = Encryptor::new(&encryption_key)?;
            app.insert(encryptor)?;

            // Build configured OAuth providers
            let http_client = app.get::<reqwest::Client>().context("no HTTP client")?;
            let oauth_providers = OAuthService::build_provider_map(app.config(), http_client)
                .context("build OAuth providers")?;
            app.insert(oauth_providers)?;

            // Start session cleanup task
            let db_pool = app.get::<DbPool>().context("no db pool")?.to_owned();
            tokio::spawn(async move {
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

            Ok(app)
        })
        .on_setup(|app, router| {
            // Session persistence
            let redis_store =
                RedisStore::with_prefix(app.state().redis.clone(), REDIS_PREFIX.to_owned());
            let db_store = SessionDbStore::new(app.state().db_pool.clone());
            let session_store = CachingSessionStore::new(redis_store, db_store);

            // Add session / cookie management to router
            let session_layer = SessionManagerLayer::new(session_store)
                .with_name(app.config().auth.cookie_name.clone())
                .with_expiry(Expiry::OnInactivity(cookie::time::Duration::minutes(15))) // default short session for login/OAuth
                .with_private(cookie::Key::derive_from(&hex::decode(
                    &app.config().auth.encryption_key,
                )?))
                .with_path("/")
                .with_secure(true)
                .with_http_only(true)
                .with_same_site(cookie::SameSite::Lax);

            Ok(router.layer(session_layer))
        })
}
