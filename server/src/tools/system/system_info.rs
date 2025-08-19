use rocket::async_trait;

use crate::{
    provider::{LlmTool, LlmToolType},
    tools::system::SystemToolConfig,
    utils::SenderWithLogging,
};

use super::{SystemTool, ToolError, ToolLog, ToolParameters, ToolResult};

const NAME_PREFIX: &str = "system";

const DATE_TIME_NAME: &str = "datetime_now";
const DATE_TIME_DESC: &str = "Get the current date and time in RFC3339 format. \
    Do not request this tool unless you specifically need the date and/or time to answer a user query.";

const SERVER_URL_NAME: &str = "server_url";
const SERVER_URL_DESC: &str = "Get the URL of the server that this chat application is running on. \
    This may be useful to help direct the user to files or other resources that are hosted on the server.";

const INFO_NAME: &str = "info";
const INFO_DESC: &str =
    "Get technical information about the server that this chat application is running on.";

/// Tool to get system information.
#[derive(Debug)]
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
                name: format!("{}:{}", NAME_PREFIX, DATE_TIME_NAME),
                description: DATE_TIME_DESC.into(),
                input_schema: serde_json::Value::Null,
                tool_type: LlmToolType::System,
            },
            LlmTool {
                tool_id,
                name: format!("{}:{}", NAME_PREFIX, SERVER_URL_NAME),
                description: SERVER_URL_DESC.into(),
                input_schema: serde_json::Value::Null,
                tool_type: LlmToolType::System,
            },
            LlmTool {
                tool_id,
                name: format!("{}:{}", NAME_PREFIX, INFO_NAME),
                description: INFO_DESC.into(),
                input_schema: serde_json::Value::Null,
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
        &serde_json::Value::Null
    }

    async fn execute(
        &self,
        tool_name: &str,
        _params: &ToolParameters,
        _tx: &SenderWithLogging<ToolLog>,
    ) -> ToolResult<String> {
        match tool_name.split(':').nth(1) {
            Some(DATE_TIME_NAME) => {
                let now = chrono::Utc::now();
                Ok(now.to_rfc3339())
            }
            Some(SERVER_URL_NAME) => {
                let server_url = std::env::var("RS_CHAT_SERVER_ADDRESS").map_err(|_| {
                    ToolError::ToolExecutionError("Could not determine Server URL".into())
                })?;
                Ok(server_url)
            }
            Some(INFO_NAME) => {
                let os = std::env::consts::OS.to_string();
                let arch = std::env::consts::ARCH.to_string();
                let family = std::env::consts::FAMILY.to_string();
                let info = format!("OS: {}, Arch: {}, Family: {}", os, arch, family);
                Ok(info)
            }
            _ => Err(ToolError::ToolNotFound),
        }
    }
}
