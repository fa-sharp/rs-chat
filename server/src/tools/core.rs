//! Core types and interfaces for tools

use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Standard result type for all tool operations
pub type ToolResult<T> = Result<T, ToolError>;

/// Tool logging stream chunk
#[derive(Debug, Clone)]
pub enum ToolLog {
    Result(String),
    Log(String),
    Debug(String),
    Error(String),
}

impl From<ToolLog> for rocket::response::stream::Event {
    fn from(chunk: ToolLog) -> Self {
        match chunk {
            ToolLog::Result(data) => Self::data(data).event("result"),
            ToolLog::Log(data) => Self::data(data).event("log"),
            ToolLog::Debug(data) => Self::data(data).event("debug"),
            ToolLog::Error(data) => Self::data(data).event("error"),
        }
    }
}

/// The format of the tool response
#[derive(Default, Debug, JsonSchema, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolResponseFormat {
    #[default]
    Text,
    Json,
    Markdown,
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
