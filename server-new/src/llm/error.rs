use crate::services::stream::error::StreamingError;

/// Errors that can occur in an LLM provider request
#[derive(Debug, thiserror::Error)]
pub enum LlmRequestError {
    /// Provider error message with optional request ID
    #[error("{0}")]
    Provider(String, Option<String>),
    #[error("Failed to read response: {0}")]
    Read(#[from] reqwest::Error),
    #[error("No content")]
    NoContent,
}
impl LlmRequestError {
    pub fn req_id(&self) -> Option<&str> {
        match self {
            LlmRequestError::Provider(_, req_id) => req_id.as_deref(),
            _ => None,
        }
    }
}

/// Errors that can occur in an LLM stream chunk
#[derive(Debug, thiserror::Error)]
pub enum LlmStreamChunkError {
    #[error("Provider error: {0}")]
    Provider(String),
    #[error("Failed to parse event: {0}")]
    Parsing(#[from] serde_json::Error),
    #[error("Failed to decode line: {0}")]
    Decoding(#[from] tokio_util::codec::LinesCodecError),
    #[error(transparent)]
    Streaming(#[from] StreamingError),
    #[error("Stream was cancelled")]
    StreamCancelled,
}
