//! LLM providers API

pub mod anthropic;
pub mod lorem;
pub mod openai;

use std::pin::Pin;

use dyn_clone::DynClone;
use rocket::{async_trait, futures::Stream};
use schemars::JsonSchema;
use uuid::Uuid;

use crate::{
    db::models::{ChatRsMessage, ChatRsProviderType, ChatRsToolCall},
    provider::{anthropic::AnthropicProvider, lorem::LoremProvider, openai::OpenAIProvider},
    provider_models::LlmModel,
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
    #[error("models.dev error: {0}")]
    ModelsDevError(String),
    #[error("No chat response")]
    NoResponse,
    #[error("Unsupported provider")]
    UnsupportedProvider,
    #[error("Already streaming a response for this session")]
    AlreadyStreaming,
    #[error("No stream found, or the stream was cancelled")]
    StreamNotFound,
    #[error("Missing event in stream")]
    NoStreamEvent,
    #[error("Client disconnected")]
    ClientDisconnected,
    #[error("Encryption error")]
    EncryptionError,
    #[error("Decryption error")]
    DecryptionError,
    #[error("Redis error: {0}")]
    Redis(#[from] fred::error::Error),
}

/// A streaming chunk of data from the LLM provider
#[derive(Default)]
pub struct LlmStreamChunk {
    pub text: Option<String>,
    pub tool_calls: Option<Vec<ChatRsToolCall>>,
    pub usage: Option<LlmUsage>,
}

/// Usage stats from the LLM provider
#[derive(Debug, Default, JsonSchema, serde::Serialize, serde::Deserialize)]
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

/// Generic tool that can be passed to LLM providers
#[derive(Debug)]
pub struct LlmTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    /// ID of the RsChat tool that this is derived from
    pub tool_id: Uuid,
    /// The type of tool this is derived from (internal, external API, etc.)
    pub tool_type: LlmToolType,
}

#[derive(Default, Debug, Clone, Copy, JsonSchema, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmToolType {
    #[default]
    System,
    ExternalApi,
}

/// Unified API for LLM providers
#[async_trait]
pub trait LlmApiProvider: Send + Sync + DynClone {
    /// Stream a chat response from the provider
    async fn chat_stream(
        &self,
        messages: Vec<ChatRsMessage>,
        tools: Option<Vec<LlmTool>>,
        options: &LlmApiProviderSharedOptions,
    ) -> Result<LlmApiStream, LlmError>;

    /// Submit a prompt to the provider (not streamed)
    async fn prompt(
        &self,
        message: &str,
        options: &LlmApiProviderSharedOptions,
    ) -> Result<String, LlmError>;

    /// List available models from the provider
    async fn list_models(&self) -> Result<Vec<LlmModel>, LlmError>;
}

/// Build the LLM API to make calls to the provider
pub fn build_llm_provider_api(
    provider_type: &ChatRsProviderType,
    base_url: Option<&str>,
    api_key: Option<&str>,
    http_client: &reqwest::Client,
    redis: &fred::prelude::Pool,
) -> Result<Box<dyn LlmApiProvider>, LlmError> {
    match provider_type {
        ChatRsProviderType::Openai => Ok(Box::new(OpenAIProvider::new(
            http_client,
            redis,
            api_key.ok_or(LlmError::MissingApiKey)?,
            base_url,
        ))),
        ChatRsProviderType::Anthropic => Ok(Box::new(AnthropicProvider::new(
            http_client,
            redis,
            api_key.ok_or(LlmError::MissingApiKey)?,
        ))),
        ChatRsProviderType::Lorem => Ok(Box::new(LoremProvider::new())),
    }
}
