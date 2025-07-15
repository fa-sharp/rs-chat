use rocket::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::tools::{
    web_search::{WebSearchResult, WebSearchToolProvider},
    ChatRsToolError,
};

#[derive(Debug, JsonSchema, Serialize, Deserialize)]
pub struct SerpApiConfig {
    pub api_key: String,
    /// Country code for search results. See https://serpapi.com/google-countries
    #[serde(default)]
    pub country: Option<String>,
    /// Language code for search results. See https://serpapi.com/google-lr-languages
    #[serde(default)]
    pub search_lang: Option<String>,
}

pub struct SerpApiSearchTool<'a> {
    config: &'a SerpApiConfig,
    count: u8,
}
impl<'a> SerpApiSearchTool<'a> {
    pub fn new(config: &'a SerpApiConfig, count: u8) -> Self {
        Self { config, count }
    }
}
#[async_trait]
impl<'a> WebSearchToolProvider for SerpApiSearchTool<'a> {
    fn name(&self) -> &str {
        "SerpApi"
    }

    async fn search(
        &self,
        http_client: &reqwest::Client,
        query: &str,
    ) -> Result<Vec<WebSearchResult>, ChatRsToolError> {
        let mut url = "https://serpapi.com/search.json".to_string();
        url.push_str(&format!(
            "?engine=google&q={}&api_key={}",
            urlencoding::encode(query),
            urlencoding::encode(&self.config.api_key)
        ));
        url.push_str(&format!("&num={}", self.count));

        if let Some(location) = &self.config.country {
            url.push_str(&format!("&gl={}", urlencoding::encode(location)));
        }

        if let Some(search_lang) = &self.config.search_lang {
            url.push_str(&format!("&hl={}", search_lang));
        }

        let response = http_client.get(&url).send().await.map_err(|e| {
            ChatRsToolError::ToolExecutionError(format!("SerpAPI search failed: {}", e))
        })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ChatRsToolError::ToolExecutionError(format!(
                "SerpAPI error {}: {}",
                status, error_text
            )));
        }

        let serp_response: SerpApiResponse = response.json().await.map_err(|e| {
            ChatRsToolError::ToolExecutionError(format!("Failed to parse SerpAPI response: {}", e))
        })?;

        Ok(serp_response
            .organic_results
            .into_iter()
            .map(|result| WebSearchResult {
                title: result.title,
                url: result.link,
                description: result.snippet.unwrap_or_default(),
            })
            .collect())
    }
}

#[derive(Debug, Deserialize)]
struct SerpApiResponse {
    organic_results: Vec<SerpApiResult>,
}

#[derive(Debug, Deserialize)]
struct SerpApiResult {
    title: String,
    link: String,
    snippet: Option<String>,
}
