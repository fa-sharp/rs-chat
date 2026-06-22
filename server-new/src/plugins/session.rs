use anyhow::{Context, bail};
use axum_plugin::AdHocPlugin;
use tower_sessions::{
    Expiry, SessionManagerLayer,
    cookie::{Key, SameSite, time::Duration},
};

use crate::state::AppState;

pub fn plugin() -> AdHocPlugin<AppState> {
    AdHocPlugin::named("Session").on_setup(|router, state: &AppState| {
        let cookie_key =
            hex::decode(&state.config.cookie_key).context("cookie_key must be hex value")?;
        if cookie_key.len() < 32 {
            bail!("cookie_key must be at least 32 bytes");
        }

        // TODO change to a persistent store!
        let session_store = tower_sessions::MemoryStore::default();
        let session_layer = SessionManagerLayer::new(session_store)
            .with_name(state.config.cookie_name.clone())
            .with_expiry(Expiry::OnInactivity(Duration::seconds(
                state.config.session_length,
            )))
            .with_private(Key::derive_from(&cookie_key))
            .with_path("/")
            .with_secure(true)
            .with_http_only(true)
            .with_same_site(SameSite::Lax);

        Ok(router.layer(session_layer))
    })
}
