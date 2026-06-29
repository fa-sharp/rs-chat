use crate::services::llm::error::LlmRequestError;

/// Chat service errors
#[derive(Debug, thiserror::Error)]
pub enum ChatError {
    #[error(transparent)]
    Request(#[from] LlmRequestError),
}
