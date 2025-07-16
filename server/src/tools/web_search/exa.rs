use rocket::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::tools::{
    utils::http_request_builder::HttpRequestBuilder,
    web_search::{WebSearchProvider, WebSearchResult},
    ToolError,
};

#[derive(Debug, JsonSchema, Serialize, Deserialize)]
pub struct ExaConfig {
    pub api_key: String,
}

pub struct ExaSearchTool<'a> {
    config: &'a ExaConfig,
    count: u8,
}
impl<'a> ExaSearchTool<'a> {
    pub fn new(config: &'a ExaConfig, count: u8) -> Self {
        Self { config, count }
    }
}
#[async_trait]
impl<'a> WebSearchProvider for ExaSearchTool<'a> {
    fn name(&self) -> &str {
        "exa.ai"
    }

    async fn search(
        &self,
        http_client: &reqwest::Client,
        query: &str,
    ) -> Result<Vec<WebSearchResult>, ToolError> {
        let builder = HttpRequestBuilder::new("GET", "https://api.exa.ai/search")
            .header("Authorization", &format!("Bearer {}", &self.config.api_key))?
            .body(
                serde_json::json!({
                    "query": query,
                    "type": "auto",
                    "numResults": self.count,
                    "contents": {
                        "text": true
                    }
                })
                .to_string(),
            );

        let response_text = builder.send(http_client).await?;
        let exa_response: ExaSearchResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                ToolError::ToolExecutionError(format!("Failed to parse Exa response: {}", e))
            })?;

        Ok(exa_response
            .results
            .into_iter()
            .map(|result| WebSearchResult {
                title: result.title,
                url: result.url,
                description: result.text.unwrap_or_default(),
            })
            .collect())
    }
}

#[derive(Debug, Deserialize)]
struct ExaSearchResponse {
    results: Vec<ExaSearchResult>,
}

#[derive(Debug, Deserialize)]
struct ExaSearchResult {
    title: String,
    url: String,
    text: Option<String>,
}
