use reqwest::header::{HeaderMap, HeaderValue};
use rocket::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::tools::{
    web_search::{WebSearchResult, WebSearchToolProvider},
    ChatRsToolError,
};

#[derive(Debug, JsonSchema, Serialize, Deserialize)]
pub struct BraveConfig {
    pub api_key: String,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub search_lang: Option<String>,
}

pub struct BraveSearchTool<'a> {
    config: &'a BraveConfig,
    count: u8,
}
impl<'a> BraveSearchTool<'a> {
    pub fn new(config: &'a BraveConfig, count: u8) -> Self {
        Self { config, count }
    }
}
#[async_trait]
impl<'a> WebSearchToolProvider for BraveSearchTool<'a> {
    fn name(&self) -> &str {
        "Brave"
    }

    async fn search(
        &self,
        http_client: &reqwest::Client,
        query: &str,
    ) -> Result<Vec<WebSearchResult>, ChatRsToolError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Subscription-Token",
            HeaderValue::from_str(&self.config.api_key).map_err(|_| {
                ChatRsToolError::FormattingError("Invalid API key format".to_string())
            })?,
        );
        headers.insert("Accept", HeaderValue::from_static("application/json"));

        let mut url = "https://api.search.brave.com/res/v1/web/search".to_string();
        url.push_str(&format!("?q={}", urlencoding::encode(query)));
        url.push_str(&format!("&count={}", self.count));
        url.push_str("&text_decorations=false");

        if let Some(country) = &self.config.country {
            url.push_str(&format!("&country={}", country));
        }

        if let Some(search_lang) = &self.config.search_lang {
            url.push_str(&format!("&search_lang={}", search_lang));
        }

        let response = http_client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| {
                ChatRsToolError::ToolExecutionError(format!("Brave search failed: {}", e))
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ChatRsToolError::ToolExecutionError(format!(
                "Brave API error {}: {}",
                status, error_text
            )));
        }

        let brave_response: BraveSearchResponse = response.json().await.map_err(|e| {
            ChatRsToolError::ToolExecutionError(format!("Failed to parse Brave response: {}", e))
        })?;

        Ok(brave_response
            .web
            .results
            .into_iter()
            .map(|result| WebSearchResult {
                title: result.title,
                url: result.url,
                description: result.description.unwrap_or_default(),
            })
            .collect())
    }
}

#[derive(Debug, Deserialize)]
struct BraveSearchResponse {
    web: BraveWebResults,
}

#[derive(Debug, Deserialize)]
struct BraveWebResults {
    results: Vec<BraveSearchResult>,
}

#[derive(Debug, Deserialize)]
struct BraveSearchResult {
    title: String,
    url: String,
    description: Option<String>,
}
