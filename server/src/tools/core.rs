//! Core types and interfaces for tools

use std::collections::HashMap;

use rocket::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;

/// Standard result type for all tool operations
pub type ToolResult<T> = Result<T, ToolError>;

/// Tool response stream chunk
#[derive(Debug)]
pub enum ToolMessageChunk {
    Result(String),
    Log(String),
    Debug(String),
    Error(String),
}

impl From<ToolMessageChunk> for rocket::response::stream::Event {
    fn from(chunk: ToolMessageChunk) -> Self {
        match chunk {
            ToolMessageChunk::Result(data) => Self::data(data).event("result"),
            ToolMessageChunk::Log(data) => Self::data(data).event("log"),
            ToolMessageChunk::Debug(data) => Self::data(data).event("debug"),
            ToolMessageChunk::Error(data) => Self::data(data).event("error"),
        }
    }
}

/// Tool input parameters
pub type ToolParameters = HashMap<String, serde_json::Value>;

/// Tool-related errors
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("Invalid JSON schema: {0}")]
    InvalidJsonSchema(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
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
    #[error("Tool execution cancelled: {0}")]
    Cancelled(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
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

    /// Execute the tool with given parameters. Real-time logs/status updates can be sent via the sender.
    async fn execute(
        &self,
        parameters: &ToolParameters,
        sender: Sender<ToolMessageChunk>,
    ) -> ToolResult<String>;
}

/// JSON schema for tool input parameters
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ToolJsonSchema {
    pub r#type: ToolJsonSchemaType,
    pub properties: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    #[serde(rename = "additionalProperties")]
    pub additional_properties: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub enum ToolJsonSchemaType {
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
