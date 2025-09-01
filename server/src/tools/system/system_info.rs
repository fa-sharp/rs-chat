use std::sync::LazyLock;

use rocket::async_trait;
use schemars::JsonSchema;

use crate::{
    config::AppConfig,
    provider::LlmToolType,
    tools::{system::SystemToolConfig, utils::get_json_schema},
    utils::SenderWithLogging,
};

use super::*;

const TOOL_PREFIX: &str = "system_";

static JSON_SCHEMA: LazyLock<serde_json::Value> =
    LazyLock::new(|| get_json_schema::<SystemInfoConfig>());

const DATE_TIME_NAME: &str = "datetime_now";
const DATE_TIME_DESC: &str = "Get the current date and time in RFC3339 format. \
    Do not request this tool unless you specifically need the date and/or time to answer a user query.";

const SERVER_URL_NAME: &str = "server_url";
const SERVER_URL_DESC: &str = "Get the URL of the server that this chat application is running on. \
    This may be useful to help direct the user to files or other resources that are hosted on the server.";

/// Tool to get system information.
pub struct SystemInfo<'a> {
    app_config: &'a AppConfig,
}
impl<'a> SystemInfo<'a> {
    pub fn new(app_config: &'a AppConfig) -> Self {
        SystemInfo { app_config }
    }
}

#[derive(Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SystemInfoConfig {}

impl SystemToolConfig for SystemInfoConfig {
    type DynamicConfig = ();

    fn get_llm_tools(
        &self,
        tool_id: uuid::Uuid,
        _input_config: Option<&Self::DynamicConfig>,
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
impl SystemTool for SystemInfo<'_> {
    fn input_schema(&self, _tool_name: &str) -> ToolResult<&serde_json::Value> {
        Ok(&JSON_SCHEMA)
    }

    async fn execute(
        &mut self,
        tool_name: &str,
        _params: serde_json::Value,
        _tx: &SenderWithLogging<ToolLog>,
    ) -> ToolResult<(String, ToolResponseFormat)> {
        match tool_name.strip_prefix(TOOL_PREFIX) {
            Some(DATE_TIME_NAME) => {
                let now = chrono::Utc::now();
                Ok((now.to_rfc3339(), ToolResponseFormat::Text))
            }
            Some(SERVER_URL_NAME) => Ok((
                self.app_config.server_address.clone(),
                ToolResponseFormat::Text,
            )),
            _ => Err(ToolError::ToolNotFound),
        }
    }
}
