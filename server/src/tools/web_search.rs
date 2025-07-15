use std::collections::HashMap;

use reqwest::header::{HeaderMap, HeaderValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::tools::ChatRsToolError;

pub struct WebSearchTool<'a> {
    http_client: reqwest::Client,
    config: &'a WebSearchToolData,
}

#[derive(Debug, JsonSchema, Serialize, Deserialize)]
pub struct WebSearchToolData {
    pub api_key: String,
    #[serde(default = "default_count")]
    pub count: u8,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub search_lang: Option<String>,
}

fn default_count() -> u8 {
    10
}

type Parameters = HashMap<String, serde_json::Value>;

impl<'a> WebSearchTool<'a> {
    pub fn new(http_client: &reqwest::Client, config: &'a WebSearchToolData) -> Self {
        Self {
            http_client: http_client.clone(),
            config,
        }
    }

    pub async fn execute_tool(&self, parameters: &Parameters) -> Result<String, ChatRsToolError> {
        let query = parameters
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ChatRsToolError::FormattingError("Missing 'query' parameter".to_string())
            })?;

        if query.trim().is_empty() {
            return Err(ChatRsToolError::FormattingError(
                "Query parameter cannot be empty".to_string(),
            ));
        }

        rocket::info!("Web Search Tool: executing search query");
        let search_results = self.search(query).await?;
        let formatted_results = self.format_results(&search_results);
        rocket::info!(
            "Web Search Tool: got {} results",
            search_results.web.results.len()
        );

        Ok(formatted_results)
    }

    async fn search(&self, query: &str) -> Result<BraveSearchResponse, ChatRsToolError> {
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
        url.push_str(&format!("&count={}", self.config.count));
        url.push_str("&text_decorations=false");

        // Add country parameter if specified
        if let Some(country) = &self.config.country {
            url.push_str(&format!("&country={}", country));
        }

        // Add search language parameter if specified
        if let Some(search_lang) = &self.config.search_lang {
            url.push_str(&format!("&search_lang={}", search_lang));
        }

        let response = self
            .http_client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| {
                ChatRsToolError::ToolExecutionError(format!("Search request failed: {}", e))
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ChatRsToolError::ToolExecutionError(format!(
                "Search API error {}: {}",
                status, error_text
            )));
        }

        let search_response: BraveSearchResponse = response.json().await.map_err(|e| {
            ChatRsToolError::ToolExecutionError(format!("Failed to parse search response: {}", e))
        })?;

        Ok(search_response)
    }

    fn format_results(&self, results: &BraveSearchResponse) -> String {
        if results.web.results.is_empty() {
            return "No search results found.".to_string();
        }

        let mut formatted = String::new();
        formatted.push_str(&format!(
            "Search Results ({})\n\n",
            results.web.results.len()
        ));

        for (i, result) in results.web.results.iter().enumerate().take(5) {
            formatted.push_str(&format!("{}. {}\n", i + 1, result.title));
            formatted.push_str(&format!("   URL: {}\n", result.url));
            if let Some(description) = &result.description {
                // Truncate long descriptions
                let truncated = if description.len() > 200 {
                    format!("{}...", &description[..200])
                } else {
                    description.clone()
                };
                formatted.push_str(&format!("   {}\n", truncated));
            }
            formatted.push('\n');
        }

        // Add note if there are more results
        if results.web.results.len() > 5 {
            formatted.push_str(&format!(
                "... and {} more results\n",
                results.web.results.len() - 5
            ));
        }

        formatted
    }
}

#[derive(Debug, Deserialize)]
struct BraveSearchResponse {
    web: WebResults,
}

#[derive(Debug, Deserialize)]
struct WebResults {
    results: Vec<SearchResult>,
}

#[derive(Debug, Deserialize)]
struct SearchResult {
    title: String,
    url: String,
    description: Option<String>,
}
