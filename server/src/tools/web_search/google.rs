use rocket::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::tools::{
    web_search::{WebSearchResult, WebSearchProvider},
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
        let mut url = "https://www.googleapis.com/customsearch/v1".to_string();
        url.push_str(&format!(
            "?key={}&cx={}&q={}",
            urlencoding::encode(&self.config.api_key),
            urlencoding::encode(&self.config.cx),
            urlencoding::encode(query)
        ));
        url.push_str(&format!("&num={}", self.count.min(10))); // Google CSE max is 10

        if let Some(search_lang) = &self.config.search_lang {
            url.push_str(&format!("&lr={}", search_lang));
        }

        if let Some(country) = &self.config.country {
            url.push_str(&format!("&gl={}", country));
        }

        let response =
            http_client.get(&url).send().await.map_err(|e| {
                ToolError::ToolExecutionError(format!("Google search failed: {}", e))
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ToolError::ToolExecutionError(format!(
                "Google API error {}: {}",
                status, error_text
            )));
        }

        let google_response: GoogleSearchResponse = response.json().await.map_err(|e| {
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
