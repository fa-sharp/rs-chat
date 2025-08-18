mod core;
mod external_api;
mod system;
mod utils;

pub use {
    core::{ToolError, ToolJsonSchema, ToolLog, ToolParameters, ToolResult},
    external_api::{ChatRsExternalApiToolConfig, ExternalApiToolInput},
    system::{ChatRsSystemToolConfig, SystemToolInput},
};

use schemars::JsonSchema;

/// User configuration of tools when sending a chat message
#[derive(Debug, Default, PartialEq, JsonSchema, serde::Serialize, serde::Deserialize)]
pub struct SendChatToolInput {
    pub system: Option<SystemToolInput>,
    pub external_apis: Option<Vec<ExternalApiToolInput>>,
}
