/// Chat service errors
#[derive(Debug, thiserror::Error)]
pub enum ChatError {
    #[error(transparent)]
    Request(#[from] crate::llm::error::LlmRequestError),
    #[error("Invalid message history")]
    Messages,
    #[error(transparent)]
    Streaming(#[from] crate::services::stream::error::StreamingError),
    #[error("database error: {0}")]
    Database(#[from] diesel::result::Error),
    #[error("database pool error: {0}")]
    DatabasePool(#[from] crate::db::DbPoolError),
}
