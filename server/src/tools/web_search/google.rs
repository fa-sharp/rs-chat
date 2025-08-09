use rocket::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::tools::{
    utils::http_request_builder::HttpRequestBuilder,
    web_search::{WebSearchProvider, WebSearchResult},
    ToolError,
};

#[derive(Debug, JsonSchema, Serialize, Deserialize)]
pub struct GoogleCustomSearchConfig {
    pub api_key: String,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub search_lang: Option<String>,
    /// Google Custom Search Engine ID
    pub cx: String,
}

pub struct GoogleCustomSearchTool<'a> {
    config: &'a GoogleCustomSearchConfig,
    count: u8,
}
impl<'a> GoogleCustomSearchTool<'a> {
    pub fn new(config: &'a GoogleCustomSearchConfig, count: u8) -> Self {
        Self { config, count }
    }
}
#[async_trait]
impl<'a> WebSearchProvider for GoogleCustomSearchTool<'a> {
    fn name(&self) -> &str {
        "Google Custom Search"
    }

    async fn search(
        &self,
        http_client: &reqwest::Client,
        query: &str,
    ) -> Result<Vec<WebSearchResult>, ToolError> {
        let mut builder =
            HttpRequestBuilder::new("GET", "https://www.googleapis.com/customsearch/v1")
                .query_param("key", &self.config.api_key)
                .query_param("cx", &self.config.cx)
                .query_param("q", query)
                .query_param("num", &self.count.min(10).to_string()); // Google CSE max is 10
        if let Some(search_lang) = &self.config.search_lang {
            builder = builder.query_param("lr", search_lang);
        }
        if let Some(country) = &self.config.country {
            builder = builder.query_param("gl", country);
        }

        let response = builder.send(http_client).await?;
        let google_response: GoogleSearchResponse =
            serde_json::from_str(&response).map_err(|e| {
                ToolError::ToolExecutionError(format!("Failed to parse Google response: {}", e))
            })?;

        Ok(google_response
            .items
            .unwrap_or_default()
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
struct GoogleSearchResponse {
    items: Option<Vec<GoogleSearchResult>>,
}

#[derive(Debug, Deserialize)]
struct GoogleSearchResult {
    title: String,
    link: String,
    snippet: Option<String>,
}
