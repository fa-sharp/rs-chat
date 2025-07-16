use rocket::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::tools::{
    utils::http_request_builder::HttpRequestBuilder,
    web_search::{WebSearchProvider, WebSearchResult},
    ToolError,
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
    /// Search engine to use. See https://serpapi.com/search-api. Default: "google"
    #[serde(default)]
    pub engine: Option<String>,
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
impl<'a> WebSearchProvider for SerpApiSearchTool<'a> {
    fn name(&self) -> &str {
        "SerpApi"
    }

    async fn search(
        &self,
        http_client: &reqwest::Client,
        query: &str,
    ) -> Result<Vec<WebSearchResult>, ToolError> {
        let mut builder = HttpRequestBuilder::new("GET", "https://serpapi.com/search.json")
            .query_param("engine", self.config.engine.as_deref().unwrap_or("google"))
            .query_param("q", query)
            .query_param("api_key", &self.config.api_key)
            .query_param("num", &self.count.to_string());
        if let Some(location) = &self.config.country {
            builder = builder.query_param("gl", location);
        }
        if let Some(search_lang) = &self.config.search_lang {
            builder = builder.query_param("hl", search_lang);
        }

        let response = builder.send(http_client).await?;
        let serp_response: SerpApiResponse = serde_json::from_str(&response).map_err(|e| {
            ToolError::ToolExecutionError(format!("Failed to parse SerpAPI response: {}", e))
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
