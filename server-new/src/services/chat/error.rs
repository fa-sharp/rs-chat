use crate::error::AppError;

/// Chat service errors
#[derive(Debug, thiserror::Error)]
pub enum ChatError {
    #[error("invalid message history")]
    Messages,
    #[error("session not found")]
    SessionNotFound,
    #[error("no assistant response")]
    NoAssistantResponse,
    #[error("already streaming a response")]
    AlreadyStreaming,
    #[error(transparent)]
    Request(#[from] crate::llm::error::LlmRequestError),
    #[error(transparent)]
    Streaming(#[from] crate::services::stream::error::StreamingError),
    #[error("database error: {0}")]
    Database(#[from] diesel::result::Error),
    #[error("database pool error: {0}")]
    DatabasePool(#[from] crate::db::DbPoolError),
}

impl From<ChatError> for AppError {
    fn from(value: ChatError) -> Self {
        match value {
            ChatError::Messages => Self::bad_request("invalid messages"),
            ChatError::SessionNotFound => Self::not_found("chat session not found"),
            ChatError::NoAssistantResponse => Self::bad_request("no assistant response found"),
            ChatError::AlreadyStreaming => Self::bad_request("already streaming this chat session"),
            ChatError::Request(err) => Self::bad_request(err.to_string()),
            err => Self::internal(err.into()),
        }
    }
}
