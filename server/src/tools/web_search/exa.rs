use reqwest::header::{HeaderMap, HeaderValue};
use rocket::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::tools::{
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
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {}", self.config.api_key))
                .map_err(|_| ToolError::FormattingError("Invalid API key format".to_string()))?,
        );
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));

        let request_body = serde_json::json!({
            "query": query,
            "type": "auto",
            "numResults": self.count,
            "contents": {
                "text": true
            }
        });

        let response = http_client
            .post("https://api.exa.ai/search")
            .headers(headers)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ToolError::ToolExecutionError(format!("Exa search failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ToolError::ToolExecutionError(format!(
                "Exa API error {}: {}",
                status, error_text
            )));
        }

        let exa_response: ExaSearchResponse = response.json().await.map_err(|e| {
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
