mod code_runner;

use diesel_as_jsonb::AsJsonb;
use rocket::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    db::models::ChatRsSystemTool, provider::LlmTool, utils::sender_with_logging::SenderWithLogging,
};

use super::{ToolError, ToolLog, ToolParameters, ToolResult};

/// System tool configuration saved in the database
#[derive(Debug, Serialize, Deserialize, JsonSchema, AsJsonb)]
#[serde(tag = "type", content = "config", rename_all = "snake_case")]
pub enum ChatRsSystemToolConfig {
    CodeRunner(code_runner::CodeRunnerConfig),
    Files(()),
}

/// Chat input settings for system tools
#[derive(Debug, PartialEq, JsonSchema, Serialize, Deserialize)]
pub struct SystemToolInput {
    #[serde(default)]
    code_runner: bool,
    // TODO files, etc...
}
impl SystemToolInput {
    /// Get all the LLM tools given the user's input
    pub fn get_llm_tools(&self, system_tools: &[ChatRsSystemTool]) -> ToolResult<Vec<LlmTool>> {
        let mut llm_tools = Vec::with_capacity(1);
        if self.code_runner {
            let (config, tool_id) = system_tools
                .iter()
                .find_map(|t| match &t.config {
                    ChatRsSystemToolConfig::CodeRunner(config) => Some((config, t.id)),
                    _ => None,
                })
                .ok_or(ToolError::ToolNotFound)?;
            llm_tools.extend(config.get_llm_tools(tool_id, None));
        }
        Ok(llm_tools)
    }
}

/// Trait for all system tools which validates input parameters and executes the tool.
#[async_trait]
pub trait SystemTool: Send + Sync {
    fn input_schema(&self, tool_name: &str) -> serde_json::Value;
    async fn execute(
        &self,
        tool_name: &str,
        parameters: &ToolParameters,
        sender: &SenderWithLogging<ToolLog>,
    ) -> ToolResult<String>;

    fn validate_input_params(
        &self,
        tool_name: &str,
        parameters: &ToolParameters,
    ) -> ToolResult<()> {
        jsonschema::validate(
            &self.input_schema(tool_name),
            &serde_json::to_value(parameters)?,
        )
        .map_err(|err| ToolError::InvalidParameters(err.to_string()))
    }
}

/// Trait for configuration settings of all system tools.
trait SystemToolConfig {
    /// Dynamic configuration that can be set on every request,
    /// e.g. enable/disable features, permissions, etc.
    type DynamicConfig;

    /// Get the available LLM tools/functions for the system tool
    /// based on this configuration and any dynamic configuration.
    fn get_llm_tools(
        &self,
        tool_id: Uuid,
        input_config: Option<Self::DynamicConfig>,
    ) -> Vec<LlmTool>;

    /// Validates the configuration of the system tool.
    fn validate(&self) -> ToolResult<()> {
        Ok(())
    }
}

impl ChatRsSystemTool {
    /// Create the system tool executor from the database entity
    pub fn build_system_tool_executor(&self) -> Box<dyn SystemTool + '_> {
        match &self.config {
            ChatRsSystemToolConfig::CodeRunner(config) => {
                Box::new(code_runner::CodeRunner::new(config))
            }
            _ => unimplemented!(),
        }
    }
}
