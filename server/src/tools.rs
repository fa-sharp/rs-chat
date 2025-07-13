mod http_request;

#[derive(Debug, thiserror::Error)]
pub enum ChatRsToolError {
    #[error("Invalid tool name")]
    InvalidToolName,
    #[error("Tool not found")]
    ToolNotFound,
    #[error("Incorrect formatting")]
    IncorrectFormatting,
    #[error("Serialization error")]
    SerializationError(#[from] serde_json::Error),
    #[error("Tool execution error")]
    ToolExecutionError(String),
}
