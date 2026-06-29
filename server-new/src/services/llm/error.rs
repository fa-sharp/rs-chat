/// Errors that can occur in an LLM provider request
#[derive(Debug, thiserror::Error)]
pub enum LlmRequestError {
    #[error("Provider error: {0}")]
    Provider(String),
}

/// Errors that can occur during LLM streaming
#[derive(Debug, thiserror::Error)]
pub enum LlmStreamError {
    #[error("Provider error: {0}")]
    Provider(String),
    #[error("Failed to parse event: {0}")]
    Parsing(#[from] serde_json::Error),
    #[error("Failed to decode line: {0}")]
    Decoding(#[from] tokio_util::codec::LinesCodecError),
    #[error("Stream was cancelled")]
    StreamCancelled,
    // #[error("Redis error: {0}")]
    // Redis(#[from] fred::error::Error),
    // #[error("Tinistream error: {0}")]
    // Tinistream(#[from] crate::stream::TiniError),
    // #[error("Websocket error: {0}")]
    // Websocket(#[from] reqwest_websocket::Error),
}
