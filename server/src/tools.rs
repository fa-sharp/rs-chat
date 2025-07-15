mod http_request;

use std::collections::HashMap;

pub use http_request::{HttpRequestTool, HttpRequestToolData};

use crate::db::models::{ChatRsTool, ChatRsToolData, ChatRsToolJsonSchema};

#[derive(Debug, thiserror::Error)]
pub enum ChatRsToolError {
    #[error("Invalid tool name")]
    InvalidToolName,
    #[error("Invalid JSON schema: {0}")]
    InvalidJsonSchema(String),
    #[error("Input parameters don't match JSON schema: {0}")]
    InvalidParameters(String),
    #[error("Tool not found")]
    ToolNotFound,
    #[error("Tool call not found")]
    ToolCallNotFound,
    #[error("Formatting error: {0}")]
    FormattingError(String),
    #[error("Serialization error")]
    SerializationError(#[from] serde_json::Error),
    #[error("Tool execution error: {0}")]
    ToolExecutionError(String),
}

/// Validate the input parameters and get tool response
pub async fn get_tool_response(
    tool: &ChatRsTool,
    parameters: &HashMap<String, serde_json::Value>,
    http_client: &reqwest::Client,
) -> (String, Option<bool>) {
    if let Err(e) = validate_tool_input(&tool.input_schema, parameters) {
        (e.to_string(), Some(true))
    } else {
        match &tool.data {
            ChatRsToolData::Http(http_request_config) => {
                let http_request_tool = HttpRequestTool::new(http_client, http_request_config);
                http_request_tool
                    .execute_tool(parameters)
                    .await
                    .map_or_else(|e| (e.to_string(), Some(true)), |response| (response, None))
            }
        }
    }
}

/// Ensure input parameters match the schema.
fn validate_tool_input(
    input_schema: &ChatRsToolJsonSchema,
    parameters: &HashMap<String, serde_json::Value>,
) -> Result<(), ChatRsToolError> {
    jsonschema::validate(
        &serde_json::to_value(input_schema)?,
        &serde_json::to_value(parameters)?,
    )
    .map_err(|err| ChatRsToolError::InvalidParameters(err.to_string()))
}

/// Ensure JSON schema is valid (using Draft 2020-12).
/// Also sets `additionalProperties` to false as required by OpenAI.
pub fn validate_tool_schema(
    input_schema: &mut ChatRsToolJsonSchema,
) -> Result<(), ChatRsToolError> {
    input_schema.additional_properties = Some(false);
    jsonschema::draft202012::meta::validate(&serde_json::to_value(input_schema)?)
        .map_err(|e| ChatRsToolError::InvalidJsonSchema(e.to_string()))?;
    Ok(())
}
