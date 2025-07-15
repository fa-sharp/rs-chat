mod brave;
mod exa;
mod google;
mod serpapi;

use std::collections::HashMap;

use rocket::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::tools::{
    web_search::{
        brave::{BraveConfig, BraveSearchTool},
        exa::{ExaConfig, ExaSearchTool},
        google::{GoogleCustomSearchConfig, GoogleCustomSearchTool},
        serpapi::{SerpApiConfig, SerpApiSearchTool},
    },
    ChatRsToolError,
};

#[derive(Debug, JsonSchema, Serialize, Deserialize)]
pub struct WebSearchToolData {
    pub provider: WebSearchProviderConfig,
    #[serde(default = "default_count")]
    pub count: u8,
}

fn default_count() -> u8 {
    10
}

#[derive(Debug, JsonSchema, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum WebSearchProviderConfig {
    Brave(BraveConfig),
    SerpAPI(SerpApiConfig),
    GoogleCustomSearch(GoogleCustomSearchConfig),
    Exa(ExaConfig),
}

#[async_trait]
trait WebSearchToolProvider {
    fn name(&self) -> &str;
    async fn search(
        &self,
        http_client: &reqwest::Client,
        query: &str,
    ) -> Result<Vec<WebSearchResult>, ChatRsToolError>;
}

#[derive(Debug)]
pub struct WebSearchResult {
    pub title: String,
    pub url: String,
    pub description: String,
}

pub struct WebSearchTool<'a> {
    http_client: reqwest::Client,
    provider: Box<dyn WebSearchToolProvider + Send + Sync + 'a>,
}

impl<'a> WebSearchTool<'a> {
    pub fn new(http_client: &reqwest::Client, data: &'a WebSearchToolData) -> Self {
        let provider: Box<dyn WebSearchToolProvider + Send + Sync> = match data.provider {
            WebSearchProviderConfig::Brave(ref config) => {
                Box::new(BraveSearchTool::new(config, data.count))
            }
            WebSearchProviderConfig::SerpAPI(ref config) => {
                Box::new(SerpApiSearchTool::new(config, data.count))
            }
            WebSearchProviderConfig::GoogleCustomSearch(ref config) => {
                Box::new(GoogleCustomSearchTool::new(config, data.count))
            }
            WebSearchProviderConfig::Exa(ref config) => {
                Box::new(ExaSearchTool::new(config, data.count))
            }
        };

        Self {
            http_client: http_client.clone(),
            provider,
        }
    }

    pub async fn execute_tool(
        &self,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<String, ChatRsToolError> {
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

        rocket::info!(
            "Web Search Tool ({}): executing search",
            self.provider.name()
        );
        let search_results = self.provider.search(&self.http_client, query).await?;
        let formatted_results = self.format_results(&search_results);
        rocket::info!(
            "Web Search Tool ({}): got {} results",
            self.provider.name(),
            search_results.len()
        );

        Ok(formatted_results)
    }

    fn format_results(&self, results: &[WebSearchResult]) -> String {
        if results.is_empty() {
            return "No search results found.".to_string();
        }

        let mut formatted = String::new();
        formatted.push_str(&format!(
            "Search Results from {} ({})\n\n",
            self.provider.name(),
            results.len()
        ));

        for (i, result) in results.iter().enumerate().take(5) {
            formatted.push_str(&format!("{}. {}\n", i + 1, result.title));
            formatted.push_str(&format!("   URL: {}\n", result.url));
            if !result.description.is_empty() {
                let truncated = if result.description.len() > 200 {
                    format!("{}...", &result.description[..200])
                } else {
                    result.description.clone()
                };
                formatted.push_str(&format!("   {}\n", truncated));
            }
            formatted.push('\n');
        }

        if results.len() > 5 {
            formatted.push_str(&format!("... and {} more results\n", results.len() - 5));
        }

        formatted
    }
}
