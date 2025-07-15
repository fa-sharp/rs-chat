//! Core types and interfaces for tools

use std::collections::HashMap;

use rocket::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Standard result type for all tool operations
pub type ToolResult<T> = Result<T, ToolError>;

/// Tool input parameters
pub type ToolParameters = HashMap<String, serde_json::Value>;

/// Tool-related errors
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
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

/// Core trait that all tools must implement
#[async_trait]
pub trait Tool: Send + Sync {
    /// The tool's name for logging
    fn name(&self) -> &str;

    /// Get the JSON schema for input parameters
    fn input_schema(&self) -> serde_json::Value;

    /// Validate input parameters (default implementation uses the tool's input JSON schema)
    fn validate_input(&self, parameters: &ToolParameters) -> ToolResult<()> {
        jsonschema::validate(&self.input_schema(), &serde_json::to_value(parameters)?)
            .map_err(|err| ToolError::InvalidParameters(err.to_string()))
    }

    /// Execute the tool with given parameters
    async fn execute(&self, parameters: &ToolParameters) -> ToolResult<String>;
}

/// JSON schema for tool input parameters
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub(super) struct ToolJsonSchema {
    r#type: ToolJsonSchemaType,
    properties: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    required: Option<Vec<String>>,
    #[serde(rename = "additionalProperties")]
    additional_properties: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
enum ToolJsonSchemaType {
    #[serde(rename = "object")]
    Object,
}

/// Ensure JSON schema is valid (using Draft 2020-12).
/// Also sets `additionalProperties` to false as required by OpenAI.
pub(super) fn validate_json_schema(input_schema: &mut ToolJsonSchema) -> ToolResult<()> {
    input_schema.additional_properties = Some(false);
    jsonschema::draft202012::meta::validate(&serde_json::to_value(input_schema)?)
        .map_err(|e| ToolError::InvalidJsonSchema(e.to_string()))?;
    Ok(())
}
