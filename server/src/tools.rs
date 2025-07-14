mod http_request;

pub use http_request::{HttpRequestTool, HttpRequestToolData};

#[derive(Debug, thiserror::Error)]
pub enum ChatRsToolError {
    #[error("Invalid tool name")]
    InvalidToolName,
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
