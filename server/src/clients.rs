use std::time::Duration;

use rocket::fairing::AdHoc;

use crate::stream::TinistreamClient;

// Default timeout for HTTP requests
const TIMEOUT: Duration = Duration::from_secs(10);

/// Fairing to setup HTTP clients for external services
pub fn setup_clients() -> AdHoc {
    AdHoc::on_ignite("Clients", |rocket| async {
        let app_config = crate::config::get_app_config(&rocket);

        // Main default client for LLM provider and OAuth requests
        let main_client = reqwest::ClientBuilder::new()
            .connect_timeout(TIMEOUT)
            .build()
            .expect("Failed to build HTTP client");

        // Tinistream client for managing Redis streams
        let tini_headers = {
            let api_key = &app_config.tinistream_api_key;
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert("X-API-KEY", api_key.parse().expect("Should be valid"));
            headers
        };
        let tini_http_client = reqwest::ClientBuilder::new()
            .connect_timeout(TIMEOUT)
            .timeout(TIMEOUT) // Setting total timeout as there shouldn't be any long-running requests (for now)
            .default_headers(tini_headers)
            .build()
            .expect("Failed to build HTTP client");
        let tinistream = TinistreamClient::new(tinistream_client::Client::new_with_client(
            &app_config.tinistream_url,
            tini_http_client,
        ));

        rocket.manage(main_client).manage(tinistream)
    })
}
