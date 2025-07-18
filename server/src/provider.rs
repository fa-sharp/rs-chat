pub mod anthropic;
pub mod lorem;
pub mod openai;

use std::pin::Pin;

use rocket::{async_trait, futures::Stream};
use schemars::JsonSchema;

use crate::db::models::ChatRsMessage;

pub const DEFAULT_MAX_TOKENS: u32 = 2000;
pub const DEFAULT_TEMPERATURE: f32 = 0.7;

#[derive(Debug, thiserror::Error)]
pub enum ChatRsError {
    #[error("Missing API key")]
    MissingApiKey,
    #[error("Lorem ipsum error: {0}")]
    LoremError(&'static str),
    #[error("Anthropic error: {0}")]
    AnthropicError(String),
    #[error("OpenAI error: {0}")]
    OpenAIError(String),
    #[error("No chat response")]
    NoResponse,
    #[error("Unsupported provider")]
    UnsupportedProvider,
    #[error("Encryption error")]
    EncryptionError,
    #[error("Decryption error")]
    DecryptionError,
}

#[derive(Debug, JsonSchema, serde::Serialize, serde::Deserialize)]
pub struct ChatRsUsage {
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub cost: Option<f32>,
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
