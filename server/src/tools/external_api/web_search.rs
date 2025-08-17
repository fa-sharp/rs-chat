mod exa;

use exa::ExaSearchTool;

use rocket::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    provider::{LlmTool, LlmToolType},
    utils::sender_with_logging::SenderWithLogging,
};

use super::{
    ExternalApiTool, ExternalApiToolConfig, ToolError, ToolLog, ToolParameters, ToolResult,
};

/// A web search tool that can support multiple providers.
pub struct WebSearchTool {
    provider: Box<dyn WebSearchProvider + Send + Sync>,
}

/// Saved configuration for the web search tool.
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
#[serde(tag = "type", content = "config", rename_all = "snake_case")]
pub enum WebSearchProviderConfig {
    Exa,
}

impl WebSearchProviderConfig {
    fn provider_str(&self) -> &'static str {
        match self {
            WebSearchProviderConfig::Exa => "exa",
        }
    }
}

/// Dynamic configuration for the web search tool.
#[derive(Debug, PartialEq, JsonSchema, Serialize, Deserialize)]
pub struct WebSearchDynamicConfig {
    /// Whether search is enabled.
    search: bool,
    /// Whether content extraction is enabled.
    extract: bool,
}

impl<'a> ExternalApiToolConfig for WebSearchConfig {
    type DynamicConfig = WebSearchDynamicConfig;

    fn get_llm_tools(
        &self,
        tool_id: uuid::Uuid,
        input_config: Option<&WebSearchDynamicConfig>,
    ) -> Vec<LlmTool> {
        let mut llm_tools = Vec::with_capacity(2);
        if input_config.as_ref().map_or(true, |config| config.search) {
            llm_tools.push(LlmTool {
                name: format!("{}:search", self.provider.provider_str()),
                description: "Search the web for a given query.".into(),
                input_schema: get_query_input_schema(),
                tool_id,
                tool_type: LlmToolType::ExternalApi,
            });
        }
        if input_config.as_ref().map_or(true, |config| config.extract) {
            llm_tools.push(LlmTool {
                name: format!("{}:content", self.provider.provider_str()),
                description: "Extract content from a given URL.".into(),
                input_schema: get_content_input_schema(),
                tool_id,
                tool_type: LlmToolType::ExternalApi,
            });
        }
        llm_tools
    }
}

#[async_trait]
impl ExternalApiTool for WebSearchTool {
    fn input_schema(&self, tool_name: &str) -> ToolResult<serde_json::Value> {
        match tool_name {
            "web_search" => Ok(get_query_input_schema()),
            "web_content" => Ok(get_content_input_schema()),
            _ => Err(ToolError::ToolNotFound),
        }
    }

    async fn execute(
        &self,
        _tool_name: &str,
        parameters: &ToolParameters,
        secrets: &[String],
        http_client: &reqwest::Client,
        tx: &SenderWithLogging<ToolLog>,
    ) -> ToolResult<String> {
        let api_key = secrets
            .get(0)
            .ok_or_else(|| ToolError::InvalidConfiguration("Missing API key".into()))?;
        let query = parameters
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::FormattingError("Missing 'query' parameter".to_string()))?;
        if query.trim().is_empty() {
            return Err(ToolError::FormattingError(
                "Query parameter cannot be empty".to_string(),
            ));
        }

        let _ = tx.send(ToolLog::Log("Searching...".into())).await;
        match self.provider.search(query, api_key, http_client).await {
            Ok(search_results) => {
                let message = format!("Found {} results", search_results.len());
                let _ = tx.send(ToolLog::Log(message)).await;
                let formatted_results = self.format_results(&search_results);
                Ok(formatted_results)
            }
            Err(err) => {
                let error_message = format!("Search error: {}", err);
                let _ = tx.send(ToolLog::Error(error_message)).await;
                Err(err)
            }
        }
    }
}

impl WebSearchTool {
    pub fn new(config: &WebSearchConfig) -> Self {
        let provider: Box<dyn WebSearchProvider + Send + Sync> = match &config.provider {
            WebSearchProviderConfig::Exa => Box::new(ExaSearchTool::new(config.count)),
        };
        Self { provider }
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

/// Trait for all web search providers
#[async_trait]
trait WebSearchProvider {
    fn name(&self) -> &str;
    async fn search(
        &self,
        query: &str,
        api_key: &str,
        http_client: &reqwest::Client,
    ) -> Result<Vec<WebSearchResult>, ToolError>;
}

#[derive(Debug)]
struct WebSearchResult {
    title: String,
    url: String,
    description: String,
}

/// Input schema for query search
fn get_query_input_schema() -> serde_json::Value {
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

/// Input schema for web content/scraping
fn get_content_input_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "url": {
                "type": "string",
                "description": "The URL to scrape"
            }
        },
        "required": ["url"],
        "additionalProperties": false
    })
}
