use std::time::Duration;

use anyhow::Context;
use axum_plugin::AdHocPlugin;

use crate::{config::AppConfig, services::stream::tinistream::TinistreamClient, state::AppState};

// Default timeout for HTTP requests
const TIMEOUT: Duration = Duration::from_secs(10);

/// Setup HTTP clients for interacting with LLMs and services
pub fn plugin() -> AdHocPlugin<AppState> {
    AdHocPlugin::named("Clients").on_init(async |mut state| {
        let config = state.get::<AppConfig>().context("no config")?;

        // Main HTTP client for LLM provider and OAuth requests.
        // No total request timeout to allow for long-lived streaming responses.
        let http_client = reqwest::ClientBuilder::new()
            .connect_timeout(TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        // `tinistream` client with API key header
        let tini_http_client = reqwest::ClientBuilder::new()
            .connect_timeout(TIMEOUT)
            .timeout(TIMEOUT)
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert("X-API-KEY", config.services.streamer_api_key.parse()?);
                headers
            })
            .build()?;
        let tinistream = TinistreamClient::new(tinistream_client::Client::new_with_client(
            &config.services.streamer_url,
            tini_http_client,
        ));

        state.insert(http_client);
        state.insert(tinistream);
        Ok(state)
    })
}
