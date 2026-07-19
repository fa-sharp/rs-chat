use std::time::Duration;

use anyhow::Context;

use crate::{plugins::AxumPlugin, services::stream::tinistream::TinistreamClient};

// Default timeout for HTTP requests
const TIMEOUT: Duration = Duration::from_secs(10);

/// Setup HTTP clients for interacting with LLMs and services
pub fn plugin() -> AxumPlugin {
    AxumPlugin::named("Clients").on_init(async |mut app| {
        // Main HTTP client for LLM provider and OAuth requests.
        // No total request timeout to allow for long-lived streaming responses.
        let http_client = reqwest::ClientBuilder::new()
            .connect_timeout(TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        app.insert(http_client)?;

        // `tinistream` client with API key header
        let tini_http_client = reqwest::ClientBuilder::new()
            .connect_timeout(TIMEOUT)
            .timeout(TIMEOUT)
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert("X-API-KEY", app.config().services.streamer_api_key.parse()?);
                headers
            })
            .build()?;
        let tinistream = TinistreamClient::new(tinistream_client::Client::new_with_client(
            &app.config().services.streamer_url,
            tini_http_client,
        ));
        tinistream.ping().await.context("connect to tinistream")?;
        app.insert(tinistream)?;

        Ok(app)
    })
}
