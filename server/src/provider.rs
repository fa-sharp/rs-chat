//! LLM providers API

pub mod anthropic;
pub mod lorem;
pub mod openai;

use std::pin::Pin;

use rocket::{async_trait, futures::Stream};
use schemars::JsonSchema;

use crate::{
    db::models::{ChatRsMessage, ChatRsProviderType, ChatRsTool, ChatRsToolCall},
    provider::{anthropic::AnthropicProvider, lorem::LoremProvider, openai::OpenAIProvider},
};

pub const DEFAULT_MAX_TOKENS: u32 = 2000;
pub const DEFAULT_TEMPERATURE: f32 = 0.7;

/// LLM provider-related errors
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
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

/// A streaming chunk of data from the LLM provider
#[derive(Default)]
pub struct LlmStreamChunk {
    pub text: Option<String>,
    pub tool_calls: Option<Vec<ChatRsToolCall>>,
    pub usage: Option<LlmUsage>,
}

/// Usage stats from the LLM provider
#[derive(Debug, JsonSchema, serde::Serialize, serde::Deserialize)]
pub struct LlmUsage {
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    /// Only included by OpenRouter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<f32>,
}

/// Shared stream type for LLM providers
pub type LlmApiStream = Pin<Box<dyn Stream<Item = Result<LlmStreamChunk, LlmError>> + Send>>;

/// Shared configuration for LLM provider requests
#[derive(Clone, Debug, JsonSchema, serde::Serialize, serde::Deserialize)]
pub struct LlmApiProviderSharedOptions {
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

/// Unified API for LLM providers
#[async_trait]
pub trait LlmApiProvider {
    /// Default model to use if not specified (e.g. when generating titles)
    fn default_model(&self) -> &'static str;

    /// Stream a chat response from the provider
    async fn chat_stream(
        &self,
        messages: Vec<ChatRsMessage>,
        tools: Option<Vec<ChatRsTool>>,
        options: &LlmApiProviderSharedOptions,
    ) -> Result<LlmApiStream, LlmError>;

    /// Submit a prompt to the provider (not streamed)
    async fn prompt(
        &self,
        message: &str,
        options: &LlmApiProviderSharedOptions,
    ) -> Result<String, LlmError>;
}

/// Build the LLM API provider to make calls to the provider
pub fn build_llm_provider_api<'a>(
    provider_type: &ChatRsProviderType,
    base_url: Option<&'a str>,
    api_key: Option<&'a str>,
    http_client: &reqwest::Client,
) -> Result<Box<dyn LlmApiProvider + Send + 'a>, LlmError> {
    match provider_type {
        ChatRsProviderType::Openai => Ok(Box::new(OpenAIProvider::new(
            http_client,
            api_key.ok_or(LlmError::MissingApiKey)?,
            base_url,
        ))),
        ChatRsProviderType::Anthropic => Ok(Box::new(AnthropicProvider::new(
            http_client,
            api_key.ok_or(LlmError::MissingApiKey)?,
        ))),
        ChatRsProviderType::Lorem => Ok(Box::new(LoremProvider::new())),
    }
}
