use crate::stream::TinistreamClient;

pub fn setup_tini_client() -> TinistreamClient {
    let url =
        dotenvy::var("RS_CHAT_TINISTREAM_URL").unwrap_or("http://127.0.0.1:8081/api".to_owned());
    let api_key = dotenvy::var("RS_CHAT_TINISTREAM_API_KEY").unwrap_or("".to_owned());
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("X-API-KEY", api_key.parse().expect("Should be valid"));
    let tini_http_client = reqwest::ClientBuilder::new()
        .default_headers(headers)
        .build()
        .expect("Failed to build tinistream HTTP client");

    TinistreamClient::new(tinistream_client::Client::new_with_client(
        &url,
        tini_http_client,
    ))
}
