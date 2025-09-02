mod code_runner;
mod files;
mod system_info;

use diesel_as_jsonb::AsJsonb;
use rocket::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    config::AppConfig,
    db::{models::ChatRsSystemTool, DbConnection},
    provider::LlmTool,
    utils::SenderWithLogging,
};

use super::*;

/// System tool configuration saved in the database
#[derive(Debug, Serialize, Deserialize, JsonSchema, AsJsonb)]
#[serde(tag = "type", content = "config", rename_all = "snake_case")]
pub enum ChatRsSystemToolConfig {
    CodeRunner(code_runner::CodeRunnerConfig),
    Files(files::FilesConfig),
    SystemInfo,
}
impl ChatRsSystemToolConfig {
    /// Validate the configuration
    pub fn validate(&self) -> ToolResult<()> {
        match self {
            ChatRsSystemToolConfig::CodeRunner(config) => config.validate(),
            ChatRsSystemToolConfig::Files(config) => config.validate(),
            ChatRsSystemToolConfig::SystemInfo => Ok(()),
        }
    }
}

/// Chat input settings for system tools
#[derive(Debug, Default, PartialEq, JsonSchema, Serialize, Deserialize)]
pub struct SystemToolInput {
    /// Enable/disable the code runner tool
    #[serde(default)]
    code_runner: bool,
    /// Enable/disable tools to get system information, current date/time, etc.
    #[serde(default)]
    info: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    files: Option<files::FilesInput>,
}
impl SystemToolInput {
    /// Get all the LLM tools given the user's input
    pub fn get_llm_tools(&self, system_tools: &[ChatRsSystemTool]) -> ToolResult<Vec<LlmTool>> {
        let mut llm_tools = Vec::with_capacity(1);
        if self.code_runner {
            let (config, tool_id) = system_tools
                .iter()
                .find_map(|t| match &t.data {
                    ChatRsSystemToolConfig::CodeRunner(config) => Some((config, t.id)),
                    _ => None,
                })
                .ok_or(ToolError::ToolNotFound)?;
            llm_tools.extend(config.get_llm_tools(tool_id, None));
        }
        if self.info {
            let (config, tool_id) = system_tools
                .iter()
                .find_map(|t| match &t.data {
                    ChatRsSystemToolConfig::SystemInfo => {
                        Some((system_info::SystemInfoConfig {}, t.id))
                    }
                    _ => None,
                })
                .ok_or(ToolError::ToolNotFound)?;
            llm_tools.extend(config.get_llm_tools(tool_id, None));
        }
        if let Some(input) = &self.files {
            let (config, tool_id) = system_tools
                .iter()
                .find_map(|t| match &t.data {
                    ChatRsSystemToolConfig::Files(config) => Some((config, t.id)),
                    _ => None,
                })
                .ok_or(ToolError::ToolNotFound)?;
            llm_tools.extend(config.get_llm_tools(tool_id, Some(input)));
        }
        Ok(llm_tools)
    }
}

/// Trait for all system tools which allows validating input parameters and executing the tool.
#[async_trait]
pub trait SystemTool: Send + Sync {
    fn input_schema(&self, tool_name: &str) -> ToolResult<&serde_json::Value>;
    async fn execute(
        &mut self,
        tool_name: &str,
        parameters: serde_json::Value,
        sender: &SenderWithLogging<ToolLog>,
    ) -> ToolResult<(String, ToolResponseFormat)>;

    async fn validate_and_execute(
        &mut self,
        tool_name: &str,
        parameters: &ToolParameters,
        tx: &SenderWithLogging<ToolLog>,
    ) -> ToolResult<(String, ToolResponseFormat)> {
        let params_value = serde_json::to_value(parameters)?;
        jsonschema::validate(self.input_schema(tool_name)?, &params_value)
            .map_err(|err| ToolError::InvalidParameters(err.to_string()))?;

        self.execute(tool_name, params_value, tx).await
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
        input_config: Option<&Self::DynamicConfig>,
    ) -> Vec<LlmTool>;

    /// Validates the configuration of the system tool.
    fn validate(&self) -> ToolResult<()>;
}

impl<'a> ChatRsSystemTool {
    /// Create the system tool executor from the database entity
    pub fn build_executor(
        &'a self,
        db: &'a mut DbConnection,
        app_config: &'a AppConfig,
        session_id: &'a Uuid,
    ) -> Box<dyn SystemTool + 'a> {
        match &self.data {
            ChatRsSystemToolConfig::CodeRunner(config) => {
                Box::new(code_runner::CodeRunner::new(config))
            }
            ChatRsSystemToolConfig::SystemInfo => {
                Box::new(system_info::SystemInfo::new(app_config))
            }
            ChatRsSystemToolConfig::Files(config) => Box::new(files::Files::new(
                &self.user_id,
                session_id,
                app_config,
                db,
                config,
            )),
        }
    }
}
