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
    ExternalApiTool, ExternalApiToolConfig, ToolError, ToolLog, ToolParameters, ToolResponseFormat,
    ToolResult,
};

use exa::ExaSearchTool;

const WEB_SEARCH_NAME: &str = "web_search";
const WEB_SEARCH_DESC: &str = "Search the web for a given query.";
const EXTRACT_NAME: &str = "web_content";
const EXTRACT_DESC: &str = "Extract content from a given URL.";

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
    url: String,
}

static WEB_SEARCH_INPUT_SCHEMA: LazyLock<serde_json::Value> = LazyLock::new(|| {
    serde_json::to_value(schema_for!(QueryInputSchema)).expect("Should be valid JSON")
});
static EXTRACT_INPUT_SCHEMA: LazyLock<serde_json::Value> = LazyLock::new(|| {
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
    /// Max characters when extracting web content.
    #[serde(default = "default_max_characters")]
    #[validate(range(min = 500, max = 10_000))]
    max_characters: u32,
}
fn default_count() -> u8 {
    10
}
fn default_max_characters() -> u32 {
    5_000
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
                name: format!("{}_{}", self.provider.provider_str(), WEB_SEARCH_NAME),
                description: WEB_SEARCH_DESC.into(),
                input_schema: WEB_SEARCH_INPUT_SCHEMA.clone(),
                tool_id,
                tool_type: LlmToolType::ExternalApi,
            });
        }
        if input_config.as_ref().map_or(true, |config| config.extract) {
            llm_tools.push(LlmTool {
                name: format!("{}_{}", self.provider.provider_str(), EXTRACT_NAME),
                description: EXTRACT_DESC.into(),
                input_schema: EXTRACT_INPUT_SCHEMA.clone(),
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
            WEB_SEARCH_NAME => Ok(WEB_SEARCH_INPUT_SCHEMA.clone()),
            EXTRACT_NAME => Ok(EXTRACT_INPUT_SCHEMA.clone()),
            _ => Err(ToolError::ToolNotFound),
        }
    }

    async fn execute(
        &self,
        tool_name: &str,
        parameters: &ToolParameters,
        secrets: &[String],
        http_client: &reqwest::Client,
        tx: &SenderWithLogging<ToolLog>,
    ) -> ToolResult<(String, ToolResponseFormat)> {
        let api_key = secrets
            .get(0)
            .ok_or_else(|| ToolError::InvalidConfiguration("Missing API key".into()))?;
        match tool_name.split_once('_').ok_or(ToolError::ToolNotFound)?.1 {
            WEB_SEARCH_NAME => {
                let query = parameters
                    .get("query")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ToolError::FormattingError("Missing 'query' parameter".to_string())
                    })?
                    .trim();
                let _ = tx.send(ToolLog::Log("Searching...".into())).await;
                match self.provider.search(query, api_key, http_client).await {
                    Ok(search_results) => {
                        let message = format!("Found {} results", search_results.len());
                        let _ = tx.send(ToolLog::Log(message)).await;
                        let formatted_results = serde_json::to_string(&search_results)?;
                        Ok((formatted_results, ToolResponseFormat::Json))
                    }
                    Err(err) => {
                        let error_message = format!("Search error: {}", err);
                        let _ = tx.send(ToolLog::Error(error_message)).await;
                        Err(err)
                    }
                }
            }
            EXTRACT_NAME => {
                let url = parameters
                    .get("url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ToolError::FormattingError("Missing 'url' parameter".to_string())
                    })?
                    .trim();
                let _ = tx.send(ToolLog::Log("Extracting...".into())).await;
                match self.provider.extract(url, api_key, http_client).await {
                    Ok(extracted_text) => {
                        let message = format!("Extracted text. Length: {}", extracted_text.len());
                        let _ = tx.send(ToolLog::Log(message)).await;
                        Ok((extracted_text, ToolResponseFormat::Text))
                    }
                    Err(err) => {
                        let error_message = format!("Extraction error: {}", err);
                        let _ = tx.send(ToolLog::Error(error_message)).await;
                        Err(err)
                    }
                }
            }
            _ => Err(ToolError::ToolNotFound),
        }
    }
}

impl WebSearchTool {
    pub fn new(config: &WebSearchConfig) -> Self {
        let provider: Box<dyn WebSearchProvider + Send + Sync> = match &config.provider {
            WebSearchProviderConfig::Exa => {
                Box::new(ExaSearchTool::new(config.count, config.max_characters))
            }
        };
        Self { provider }
    }
}

/// Trait for all web search providers
#[async_trait]
trait WebSearchProvider {
    async fn search(
        &self,
        query: &str,
        api_key: &str,
        http_client: &reqwest::Client,
    ) -> ToolResult<Vec<WebSearchResult>>;
    async fn extract(
        &self,
        url: &str,
        api_key: &str,
        http_client: &reqwest::Client,
    ) -> ToolResult<String>;
}

/// Shared search result format
#[derive(Debug, Serialize)]
struct WebSearchResult {
    title: String,
    url: String,
    text: String,
}
