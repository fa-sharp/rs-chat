pub mod anthropic;
pub mod llm;
pub mod lorem;
pub mod openrouter;

use std::pin::Pin;

use ::llm::error::LLMError;
use openrouter_rs::error::OpenRouterError;
use rocket::{async_trait, futures::Stream};

use crate::db::models::ChatRsMessage;

pub const DEFAULT_MAX_TOKENS: u32 = 2000;
pub const DEFAULT_TEMPERATURE: f32 = 0.7;

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
    #[error("Anthropic error: {0}")]
    AnthropicError(String),
    #[error(transparent)]
    OpenRouterError(#[from] OpenRouterError),
    #[error("Encryption error")]
    EncryptionError,
    #[error("Decryption error")]
    DecryptionError,
}

#[derive(Debug)]
pub struct ChatRsUsage {
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
}

pub struct ChatRsStreamChunk {
    pub text: String,
    pub usage: Option<ChatRsUsage>,
}

pub type ChatRsStream = Pin<Box<dyn Stream<Item = Result<ChatRsStreamChunk, ChatRsError>> + Send>>;

/// Interface for all chat providers
#[async_trait]
pub trait ChatRsProvider {
    /// Stream a chat response given the message history
    async fn chat_stream(&self, messages: Vec<ChatRsMessage>) -> Result<ChatRsStream, ChatRsError>;

    /// Submit a prompt to the provider (not streamed)
    async fn prompt(&self, message: &str) -> Result<String, ChatRsError>;
}
