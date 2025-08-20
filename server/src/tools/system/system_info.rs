use std::sync::LazyLock;

use rocket::async_trait;
use schemars::JsonSchema;

use crate::{
    provider::{LlmTool, LlmToolType},
    tools::{system::SystemToolConfig, utils::get_json_schema},
    utils::SenderWithLogging,
};

use super::{SystemTool, ToolError, ToolLog, ToolParameters, ToolResponseFormat, ToolResult};

const TOOL_PREFIX: &str = "system_";

static JSON_SCHEMA: LazyLock<serde_json::Value> = LazyLock::new(|| get_json_schema::<SystemInfo>());

const DATE_TIME_NAME: &str = "datetime_now";
const DATE_TIME_DESC: &str = "Get the current date and time in RFC3339 format. \
    Do not request this tool unless you specifically need the date and/or time to answer a user query.";

const SERVER_URL_NAME: &str = "server_url";
const SERVER_URL_DESC: &str = "Get the URL of the server that this chat application is running on. \
    This may be useful to help direct the user to files or other resources that are hosted on the server.";

/// Tool to get system information.
#[derive(Debug, JsonSchema)]
pub struct SystemInfo {}
impl SystemInfo {
    pub fn new() -> Self {
        SystemInfo {}
    }
}

pub struct SystemInfoConfig {}
impl SystemToolConfig for SystemInfoConfig {
    type DynamicConfig = ();

    fn get_llm_tools(
        &self,
        tool_id: uuid::Uuid,
        _input_config: Option<Self::DynamicConfig>,
    ) -> Vec<LlmTool> {
        vec![
            LlmTool {
                tool_id,
                name: format!("{}{}", TOOL_PREFIX, DATE_TIME_NAME),
                description: DATE_TIME_DESC.into(),
                input_schema: JSON_SCHEMA.to_owned(),
                tool_type: LlmToolType::System,
            },
            LlmTool {
                tool_id,
                name: format!("{}{}", TOOL_PREFIX, SERVER_URL_NAME),
                description: SERVER_URL_DESC.into(),
                input_schema: JSON_SCHEMA.to_owned(),
                tool_type: LlmToolType::System,
            },
        ]
    }

    fn validate(&self) -> ToolResult<()> {
        Ok(())
    }
}

#[async_trait]
impl SystemTool for SystemInfo {
    fn input_schema(&self, _tool_name: &str) -> &serde_json::Value {
        &JSON_SCHEMA
    }

    async fn execute(
        &self,
        tool_name: &str,
        _params: &ToolParameters,
        _tx: &SenderWithLogging<ToolLog>,
    ) -> ToolResult<(String, ToolResponseFormat)> {
        match tool_name.strip_prefix(TOOL_PREFIX) {
            Some(DATE_TIME_NAME) => {
                let now = chrono::Utc::now();
                Ok((now.to_rfc3339(), ToolResponseFormat::Text))
            }
            Some(SERVER_URL_NAME) => {
                let server_url = std::env::var("RS_CHAT_SERVER_ADDRESS").map_err(|_| {
                    ToolError::ToolExecutionError("Could not determine Server URL".into())
                })?;
                Ok((server_url, ToolResponseFormat::Text))
            }
            _ => Err(ToolError::ToolNotFound),
        }
    }
}
