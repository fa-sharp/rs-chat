pub mod anthropic;
pub mod lorem;
pub mod openai;

use std::{collections::HashMap, pin::Pin};

use rocket::{async_trait, futures::Stream};
use schemars::JsonSchema;
use uuid::Uuid;

use crate::db::models::{ChatRsMessage, ChatRsTool};

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

#[derive(Default)]
pub struct ChatRsStreamChunk {
    pub text: Option<String>,
    pub tool_calls: Option<Vec<ChatRsToolCall>>,
    pub usage: Option<ChatRsUsage>,
}

/// Usage stats from provider
#[derive(Debug, JsonSchema, serde::Serialize, serde::Deserialize)]
pub struct ChatRsUsage {
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    /// Only included by OpenRouter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<f32>,
}

/// A tool call requested by the provider
#[derive(Debug, JsonSchema, serde::Serialize, serde::Deserialize)]
pub struct ChatRsToolCall {
    /// ID of the tool call
    pub id: String,
    /// ID of the tool used
    pub tool_id: Uuid,
    /// Name of the tool used
    pub tool_name: String,
    /// Input parameters passed to the tool
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Shared stream type for all providers
pub type ChatRsStream = Pin<Box<dyn Stream<Item = Result<ChatRsStreamChunk, ChatRsError>> + Send>>;

/// Interface for all chat providers
#[async_trait]
pub trait ChatRsProvider {
    /// Stream a chat response given the message history
    async fn chat_stream(
        &self,
        messages: Vec<ChatRsMessage>,
        tools: Option<Vec<ChatRsTool>>,
    ) -> Result<ChatRsStream, ChatRsError>;

    /// Submit a prompt to the provider (not streamed)
    async fn prompt(&self, message: &str) -> Result<String, ChatRsError>;
}
