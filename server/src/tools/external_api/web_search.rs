mod exa;

use std::sync::LazyLock;

use rocket::async_trait;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

use crate::{
    provider::{LlmTool, LlmToolType},
    utils::SenderWithLogging,
};

use super::{
    ExternalApiTool, ExternalApiToolConfig, ToolError, ToolLog, ToolParameters, ToolResult,
};

use exa::ExaSearchTool;

const QUERY_DESCRIPTION: &str = "Search the web for a given query.";
const CONTENT_DESCRIPTION: &str = "Extract content from a given URL.";

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct QueryInputSchema {
    /// The search query
    query: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ContentInputSchema {
    /// The URL to extract
    #[validate(url)]
    url: String,
}

static QUERY_INPUT_SCHEMA: LazyLock<serde_json::Value> = LazyLock::new(|| {
    serde_json::to_value(schema_for!(QueryInputSchema)).expect("Should be valid JSON")
});
static CONTENT_INPUT_SCHEMA: LazyLock<serde_json::Value> = LazyLock::new(|| {
    serde_json::to_value(schema_for!(ContentInputSchema)).expect("Should be valid JSON")
});

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
    #[validate(range(min = 1, max = 10))]
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
        let mut llm_tools: Vec<LlmTool> = Vec::with_capacity(2);
        if input_config.as_ref().map_or(true, |config| config.search) {
            llm_tools.push(LlmTool {
                name: format!("{}_web_search", self.provider.provider_str()),
                description: QUERY_DESCRIPTION.into(),
                input_schema: QUERY_INPUT_SCHEMA.clone(),
                tool_id,
                tool_type: LlmToolType::ExternalApi,
            });
        }
        if input_config.as_ref().map_or(true, |config| config.extract) {
            llm_tools.push(LlmTool {
                name: format!("{}_web_content", self.provider.provider_str()),
                description: CONTENT_DESCRIPTION.into(),
                input_schema: CONTENT_INPUT_SCHEMA.clone(),
                tool_id,
                tool_type: LlmToolType::ExternalApi,
            });
        }
        llm_tools
    }

    fn validate(&mut self) -> ToolResult<()> {
        let config_schema = serde_json::to_value(schema_for!(Self))?;
        jsonschema::validate(&config_schema, &serde_json::to_value(self)?)
            .map_err(|e| ToolError::InvalidConfiguration(e.to_string()))
    }
}

#[async_trait]
impl ExternalApiTool for WebSearchTool {
    fn input_schema(&self, tool_name: &str) -> ToolResult<serde_json::Value> {
        match tool_name.split_once('_').ok_or(ToolError::ToolNotFound)?.1 {
            "web_search" => Ok(QUERY_INPUT_SCHEMA.clone()),
            "web_content" => Ok(CONTENT_INPUT_SCHEMA.clone()),
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
