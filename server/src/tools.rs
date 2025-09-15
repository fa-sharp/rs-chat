mod core;
mod external_api;
mod system;
mod utils;

pub use {
    core::{ToolError, ToolJsonSchema, ToolLog, ToolParameters, ToolResponseFormat, ToolResult},
    external_api::{ChatRsExternalApiToolConfig, ExternalApiToolInput},
    system::{ChatRsSystemToolConfig, SystemToolInput},
};

use schemars::JsonSchema;
use uuid::Uuid;

use crate::{db::services::ToolDbService, errors::ApiError, provider::LlmTool};

/// User configuration of tools when sending a chat message
#[derive(Debug, Default, PartialEq, JsonSchema, serde::Serialize, serde::Deserialize)]
pub struct SendChatToolInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemToolInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_apis: Option<Vec<ExternalApiToolInput>>,
}

impl SendChatToolInput {
    /// Get all tools from the user's input in LLM generic format
    pub async fn get_llm_tools(
        &self,
        user_id: &Uuid,
        tool_db_service: &mut ToolDbService<'_>,
    ) -> Result<Vec<LlmTool>, ApiError> {
        let mut llm_tools = Vec::with_capacity(5);
        if let Some(ref system_tool_input) = self.system {
            let system_tools = tool_db_service.find_system_tools_by_user(&user_id).await?;
            let system_llm_tools = system_tool_input.get_llm_tools(&system_tools)?;
            llm_tools.extend(system_llm_tools);
        }
        if let Some(ref external_apis_input) = self.external_apis {
            let external_api_tools = tool_db_service
                .find_external_api_tools_by_user(&user_id)
                .await?;
            for tool_input in external_apis_input {
                let api_llm_tools = tool_input.into_llm_tools(&external_api_tools)?;
                llm_tools.extend(api_llm_tools);
            }
        }

        Ok(llm_tools)
    }
}
