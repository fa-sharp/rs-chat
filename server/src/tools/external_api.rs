mod custom_api;
mod web_search;

use diesel_as_jsonb::AsJsonb;
use rocket::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{db::models::ChatRsExternalApiTool, provider::LlmTool, utils::SenderWithLogging};

use super::{ToolError, ToolLog, ToolParameters, ToolResponseFormat, ToolResult};

/// External API tool configuration saved in the database
#[derive(Debug, Serialize, Deserialize, JsonSchema, AsJsonb)]
#[serde(tag = "type", content = "config", rename_all = "snake_case")]
pub enum ChatRsExternalApiToolConfig {
    CustomApi(custom_api::CustomApiConfig),
    WebSearch(web_search::WebSearchConfig),
}
impl ChatRsExternalApiToolConfig {
    /// Validate the configuration
    pub fn validate(&mut self) -> ToolResult<()> {
        match self {
            ChatRsExternalApiToolConfig::CustomApi(config) => config.validate(),
            ChatRsExternalApiToolConfig::WebSearch(config) => config.validate(),
        }
    }
}

/// Chat input settings for an external API tool
#[derive(Debug, PartialEq, JsonSchema, Serialize, Deserialize)]
pub struct ExternalApiToolInput {
    /// ID of the external API tool
    id: Uuid,
    /// Dynamic configuration for the external API tool (set permissions, features, etc.)
    config: Option<ExternalApiToolInputConfig>,
}

#[derive(Debug, PartialEq, JsonSchema, Serialize, Deserialize)]
#[serde(tag = "type", content = "config", rename_all = "snake_case")]
enum ExternalApiToolInputConfig {
    WebSearch(web_search::WebSearchDynamicConfig),
    CustomApi(custom_api::CustomApiDynamicConfig),
}

impl ExternalApiToolInput {
    /// Get all the LLM tools given the user's input
    pub fn into_llm_tools(
        &self,
        external_api_tools: &[ChatRsExternalApiTool],
    ) -> ToolResult<Vec<LlmTool>> {
        let tool = external_api_tools
            .iter()
            .find(|tool| tool.id == self.id)
            .ok_or(ToolError::ToolNotFound)?;
        let llm_tools = match &tool.data {
            ChatRsExternalApiToolConfig::CustomApi(custom_api_config) => {
                let dynamic_config = match &self.config {
                    Some(ExternalApiToolInputConfig::CustomApi(config)) => Some(config),
                    _ => None,
                };
                custom_api_config.get_llm_tools(tool.id, dynamic_config)
            }
            ChatRsExternalApiToolConfig::WebSearch(web_search_config) => {
                let dynamic_config = match &self.config {
                    Some(ExternalApiToolInputConfig::WebSearch(config)) => Some(config),
                    _ => None,
                };
                web_search_config.get_llm_tools(tool.id, dynamic_config)
            }
        };
        Ok(llm_tools)
    }
}

/// Trait for all external API tools which allows validating input and executing the tool.
#[async_trait]
pub trait ExternalApiTool: Send + Sync {
    fn input_schema(&self, tool_name: &str) -> ToolResult<serde_json::Value>;
    async fn execute(
        &self,
        tool_name: &str,
        parameters: &ToolParameters,
        secrets: &[String],
        http_client: &reqwest::Client,
        sender: &SenderWithLogging<ToolLog>,
    ) -> ToolResult<(String, ToolResponseFormat)>;

    async fn validate_and_execute(
        &self,
        tool_name: &str,
        parameters: &ToolParameters,
        secrets: &[String],
        http_client: &reqwest::Client,
        sender: &SenderWithLogging<ToolLog>,
    ) -> ToolResult<(String, ToolResponseFormat)> {
        jsonschema::validate(
            &self.input_schema(tool_name)?,
            &serde_json::to_value(parameters)?,
        )
        .map_err(|err| ToolError::InvalidParameters(err.to_string()))?;
        self.execute(tool_name, parameters, secrets, http_client, sender)
            .await
    }
}

/// Trait for configuration settings of all external API tools.
trait ExternalApiToolConfig {
    /// Dynamic configuration that can be set on every request.
    /// (e.g. permissions, enabling/disabling features, etc.)
    type DynamicConfig;

    /// Get the available LLM tools/functions for the external API
    /// based on this configuration and any dynamic configuration
    fn get_llm_tools(
        &self,
        tool_id: Uuid,
        dynamic_config: Option<&Self::DynamicConfig>,
    ) -> Vec<LlmTool>;

    /// Validates the configuration of the external API.
    fn validate(&mut self) -> ToolResult<()>;
}

impl ChatRsExternalApiTool {
    /// Create the tool executor from the database entity
    pub fn build_executor(&self) -> Box<dyn ExternalApiTool + '_> {
        match &self.data {
            ChatRsExternalApiToolConfig::CustomApi(config) => {
                Box::new(custom_api::CustomApiTool::new(config))
            }
            ChatRsExternalApiToolConfig::WebSearch(config) => {
                Box::new(web_search::WebSearchTool::new(config))
            }
        }
    }
}
