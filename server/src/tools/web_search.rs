mod brave;
mod exa;
mod google;
mod serpapi;

use rocket::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    tools::{
        core::ToolLog,
        web_search::{
            brave::{BraveConfig, BraveSearchTool},
            exa::{ExaConfig, ExaSearchTool},
            google::{GoogleCustomSearchConfig, GoogleCustomSearchTool},
            serpapi::{SerpApiConfig, SerpApiSearchTool},
        },
        Tool, ToolError, ToolParameters, ToolResult,
    },
    utils::sender_with_logging::SenderWithLogging,
};

/// A web search tool that supports multiple providers.
pub struct WebSearchTool<'a> {
    name: String,
    http_client: reqwest::Client,
    provider: Box<dyn WebSearchProvider + Send + Sync + 'a>,
}

/// Web search tool configuration.
#[derive(Debug, JsonSchema, Serialize, Deserialize)]
pub struct WebSearchConfig {
    /// Provider-specific configuration
    provider: WebSearchProviderConfig,
    /// Max search results to return.
    #[serde(default = "default_count")]
    count: u8,
}
fn default_count() -> u8 {
    10
}

#[derive(Debug, JsonSchema, Serialize, Deserialize)]
pub struct WebSearchConfigPublic {
    /// The chosen search provider.
    provider: WebSearchProviderConfigPublic,
    /// Max search results to return.
    count: u8,
}

#[derive(Debug, JsonSchema, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum WebSearchProviderConfig {
    Brave(BraveConfig),
    SerpAPI(SerpApiConfig),
    GoogleCustomSearch(GoogleCustomSearchConfig),
    Exa(ExaConfig),
}

#[derive(Debug, JsonSchema, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum WebSearchProviderConfigPublic {
    Brave,
    SerpAPI,
    GoogleCustomSearch,
    Exa,
}

impl WebSearchConfig {
    pub(super) fn validate(&self) -> ToolResult<()> {
        Ok(())
    }

    pub(super) fn get_input_schema(&self) -> serde_json::Value {
        get_input_schema()
    }
}

#[async_trait]
impl Tool for WebSearchTool<'_> {
    fn name(&self) -> &str {
        &self.name
    }

    fn input_schema(&self) -> serde_json::Value {
        get_input_schema()
    }

    async fn execute(
        &self,
        parameters: &ToolParameters,
        _sender: &SenderWithLogging<ToolLog>,
    ) -> Result<String, ToolError> {
        let query = parameters
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::FormattingError("Missing 'query' parameter".to_string()))?;
        if query.trim().is_empty() {
            return Err(ToolError::FormattingError(
                "Query parameter cannot be empty".to_string(),
            ));
        }

        let search_results = self.provider.search(&self.http_client, query).await?;
        let formatted_results = self.format_results(&search_results);

        Ok(formatted_results)
    }
}

/// Trait for all web search providers
#[async_trait]
trait WebSearchProvider {
    fn name(&self) -> &str;
    async fn search(
        &self,
        http_client: &reqwest::Client,
        query: &str,
    ) -> Result<Vec<WebSearchResult>, ToolError>;
}

#[derive(Debug)]
struct WebSearchResult {
    title: String,
    url: String,
    description: String,
}

/// Input schema for all web search tools
fn get_input_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "query": {
                "type": "string",
                "description": "The search query"
            }
        },
        "required": ["query"],
        "additionalProperties": false
    })
}

impl<'a> WebSearchTool<'a> {
    pub fn new(http_client: &reqwest::Client, data: &'a WebSearchConfig) -> Self {
        let provider: Box<dyn WebSearchProvider + Send + Sync> = match &data.provider {
            WebSearchProviderConfig::Brave(config) => {
                Box::new(BraveSearchTool::new(config, data.count))
            }
            WebSearchProviderConfig::SerpAPI(config) => {
                Box::new(SerpApiSearchTool::new(config, data.count))
            }
            WebSearchProviderConfig::GoogleCustomSearch(config) => {
                Box::new(GoogleCustomSearchTool::new(config, data.count))
            }
            WebSearchProviderConfig::Exa(config) => {
                Box::new(ExaSearchTool::new(config, data.count))
            }
        };

        Self {
            http_client: http_client.clone(),
            name: format!("Web Search ({})", provider.name()),
            provider,
        }
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

        for (i, result) in results.iter().enumerate() {
            formatted.push_str(&format!("{}. {}\n", i + 1, result.title));
            formatted.push_str(&format!("   URL: {}\n", result.url));
            if !result.description.is_empty() {
                let truncated = if result.description.len() > 300 {
                    format!("{}...", &result.description[..300])
                } else {
                    result.description.clone()
                };
                formatted.push_str(&format!("   {}\n", truncated));
            }
            formatted.push('\n');
        }

        formatted
    }
}
