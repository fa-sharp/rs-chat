//! LLM providers API

pub mod anthropic;
pub mod lorem;
pub mod models;
pub mod ollama;
pub mod openai;
mod utils;

use std::pin::Pin;

use dyn_clone::DynClone;
use rocket::{async_trait, futures::Stream};
use schemars::JsonSchema;
use uuid::Uuid;

use crate::{
    db::models::{ChatRsMessage, ChatRsProviderType, ChatRsToolCall},
    provider::{
        anthropic::AnthropicProvider, lorem::LoremProvider, models::LlmModel,
        ollama::OllamaProvider, openai::OpenAIProvider,
    },
};

pub const DEFAULT_MAX_TOKENS: u32 = 2000;
pub const DEFAULT_TEMPERATURE: f32 = 0.7;

/// LLM provider-related errors
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("Missing API key")]
    MissingApiKey,
    #[error("Provider error: {0}")]
    ProviderError(String),
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

/// LLM errors during streaming
#[derive(Debug, thiserror::Error)]
pub enum LlmStreamError {
    #[error("Provider error: {0}")]
    ProviderError(String),
    #[error("Failed to parse event: {0}")]
    Parsing(#[from] serde_json::Error),
    #[error("Failed to decode response: {0}")]
    Decoding(#[from] tokio_util::codec::LinesCodecError),
    #[error("Timeout waiting for provider response")]
    StreamTimeout,
    #[error("Stream was cancelled")]
    StreamCancelled,
    #[error("Redis error: {0}")]
    Redis(#[from] fred::error::Error),
}

/// Shared stream response type for LLM providers
pub type LlmStream = Pin<Box<dyn Stream<Item = LlmStreamChunkResult> + Send>>;

/// Shared stream chunk result type for LLM providers
pub type LlmStreamChunkResult = Result<LlmStreamChunk, LlmStreamError>;

/// A streaming chunk of data from the LLM provider
pub enum LlmStreamChunk {
    Text(String),
    ToolCalls(Vec<ChatRsToolCall>),
    PendingToolCall(LlmPendingToolCall),
    Usage(LlmUsage),
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LlmPendingToolCall {
    pub index: usize,
    pub tool_name: String,
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

/// Shared configuration for LLM provider requests
#[derive(Clone, Debug, Default, JsonSchema, serde::Serialize, serde::Deserialize)]
pub struct LlmProviderOptions {
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
        options: &LlmProviderOptions,
    ) -> Result<LlmStream, LlmError>;

    /// Submit a prompt to the provider (not streamed)
    async fn prompt(&self, message: &str, options: &LlmProviderOptions)
        -> Result<String, LlmError>;

    /// List available models from the provider
    async fn list_models(&self) -> Result<Vec<LlmModel>, LlmError>;
}

/// Build the LLM API to make calls to the provider
pub fn build_llm_provider_api(
    provider_type: &ChatRsProviderType,
    base_url: Option<&str>,
    api_key: Option<&str>,
    http_client: &reqwest::Client,
    redis: &fred::clients::Client,
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
        ChatRsProviderType::Ollama => Ok(Box::new(OllamaProvider::new(
            http_client,
            base_url.unwrap_or("http://localhost:11434"),
        ))),
        ChatRsProviderType::Lorem => Ok(Box::new(LoremProvider::new())),
    }
}
