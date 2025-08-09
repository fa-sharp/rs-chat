use rocket::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::tools::{
    utils::http_request_builder::HttpRequestBuilder,
    web_search::{WebSearchProvider, WebSearchResult},
    ToolError,
};

#[derive(Debug, JsonSchema, Serialize, Deserialize)]
pub struct BraveConfig {
    pub api_key: String,
    /// Country code for search results. See https://api-dashboard.search.brave.com/app/documentation/web-search/codes#country-codes
    #[serde(default)]
    pub country: Option<String>,
    /// Language code for search results. See https://api-dashboard.search.brave.com/app/documentation/web-search/codes#language-codes
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
impl<'a> WebSearchProvider for BraveSearchTool<'a> {
    fn name(&self) -> &str {
        "Brave"
    }

    async fn search(
        &self,
        http_client: &reqwest::Client,
        query: &str,
    ) -> Result<Vec<WebSearchResult>, ToolError> {
        let mut builder =
            HttpRequestBuilder::new("GET", "https://api.search.brave.com/res/v1/web/search")
                .header("X-Subscription-Token", &self.config.api_key)?
                .header("Accept", "application/json")?
                .query_param("q", query)
                .query_param("count", &self.count.to_string())
                .query_param("text_decorations", "false");
        if let Some(country) = &self.config.country {
            builder = builder.query_param("country", country);
        }
        if let Some(search_lang) = &self.config.search_lang {
            builder = builder.query_param("search_lang", search_lang);
        }

        let response_text = builder.send(http_client).await?;
        let brave_response: BraveSearchResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                ToolError::ToolExecutionError(format!("Failed to parse Brave response: {}", e))
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
