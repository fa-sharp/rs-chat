pub mod llm;
pub mod lorem;
pub mod openrouter;

use std::pin::Pin;

use ::llm::error::LLMError;
use openrouter_rs::error::OpenRouterError;
use rocket::{async_trait, futures::Stream};

use crate::db::models::ChatRsMessage;

#[derive(Debug, thiserror::Error)]
pub enum ChatRsError {
    #[error("Provider error: {0}")]
    ChatError(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("Unexpected database error: {0}")]
    DatabaseError(String),
    #[error("Missing API key")]
    MissingApiKey,
    #[error(transparent)]
    LlmError(#[from] LLMError),
    #[error(transparent)]
    OpenRouterError(#[from] OpenRouterError),
    #[error("Encryption error")]
    EncryptionError,
    #[error("Decryption error")]
    DecryptionError,
}

pub type ChatRsStream = Pin<Box<dyn Stream<Item = Result<String, ChatRsError>> + Send>>;

/// Interface for all chat providers
#[async_trait]
pub trait ChatRsProvider {
    /// Stream a chat response given the input and context
    async fn chat_stream(
        &self,
        input: Option<&str>,
        context: Option<Vec<ChatRsMessage>>,
    ) -> Result<ChatRsStream, ChatRsError>;

    /// Submit a prompt to the provider (not streamed)
    async fn prompt(&self, message: &str) -> Result<String, ChatRsError>;

    /// List available models for this provider
    async fn list_models(&self) -> Result<Vec<String>, ChatRsError> {
        Ok(vec![])
    }
}
