pub mod llm;
pub mod lorem;
pub mod openrouter;

use std::pin::Pin;

use ::llm::error::LLMError;
use openrouter_rs::error::OpenRouterError;
use rocket::{async_trait, futures::Stream};

use crate::db::models::{ChatRsFile, ChatRsMessage};

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

pub enum ChatRsProviderMessage {
    Message(ChatRsMessage),
    Attachment(ChatRsFile),
}

/// Interface for all chat providers
#[async_trait]
pub trait ChatRsProvider {
    /// Stream an assistant response with the provided message history
    async fn chat_stream(
        &self,
        messages: Vec<ChatRsProviderMessage>,
    ) -> Result<ChatRsStream, ChatRsError>;

    /// Submit a prompt to the provider (not streamed)
    async fn prompt(&self, message: &str) -> Result<String, ChatRsError>;
}
