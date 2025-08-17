use rocket::async_trait;
use serde::Deserialize;

use crate::tools::{utils::http_request_builder::HttpRequestBuilder, ToolError};

use super::{WebSearchProvider, WebSearchResult};

pub struct ExaSearchTool {
    count: u8,
}
impl ExaSearchTool {
    pub fn new(count: u8) -> Self {
        Self { count }
    }
}
#[async_trait]
impl WebSearchProvider for ExaSearchTool {
    fn name(&self) -> &str {
        "exa.ai"
    }

    async fn search(
        &self,
        query: &str,
        api_key: &str,
        http_client: &reqwest::Client,
    ) -> Result<Vec<WebSearchResult>, ToolError> {
        let builder = HttpRequestBuilder::new("POST", "https://api.exa.ai/search")
            .header("X-Api-Key", api_key)?
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
