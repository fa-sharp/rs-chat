//! LLM providers API

mod anthropic;
pub mod lorem;
pub mod models;
mod ollama;
mod openai;
mod utils;

use std::pin::Pin;

use dyn_clone::DynClone;
use rocket::{async_trait, futures::Stream};
use schemars::JsonSchema;
use uuid::Uuid;

use crate::{
    db::{
        models::{
            ChatRsFileType, ChatRsMessage, ChatRsMessageRole, ChatRsProviderType, ChatRsToolCall,
        },
        services::FileDbService,
        DbConnection,
    },
    errors::ApiError,
    provider::{
        anthropic::AnthropicProvider, lorem::LoremProvider, models::LlmModel,
        ollama::OllamaProvider, openai::OpenAIProvider,
    },
    storage::LocalStorage,
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
    #[error("File error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid file type: {0}")]
    InvalidFileType(String),
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

/// Stream response type for LLM providers
pub type LlmStream = Pin<Box<dyn Stream<Item = LlmStreamChunkResult> + Send>>;

/// Stream chunk result type for LLM providers
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

/// Configuration for LLM provider requests
#[derive(Clone, Debug, Default, JsonSchema, serde::Serialize, serde::Deserialize)]
pub struct LlmProviderOptions {
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

/// Generic message type to send to LLM providers
pub enum LlmMessage {
    User(LlmUserMessage),
    Assistant(LlmAssistantMessage),
    System(String),
    Tool(LlmToolResult),
}

pub struct LlmUserMessage {
    text: String,
    files: Option<Vec<LlmFileInput>>,
}

pub struct LlmFileInput {
    pub name: String,
    pub file_type: ChatRsFileType,
    pub content_type: String,
    pub content: String,
}

pub struct LlmAssistantMessage {
    text: String,
    tool_calls: Option<Vec<ChatRsToolCall>>,
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

pub struct LlmToolResult {
    tool_call_id: String,
    tool_name: String,
    content: String,
}

/// Unified API for LLM providers
#[async_trait]
pub trait LlmApiProvider: Send + Sync + DynClone {
    /// Stream a chat response from the provider
    async fn chat_stream(
        &self,
        messages: Vec<LlmMessage>,
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

/// Convert database messages to the generic messages to send to the provider implementation
pub async fn build_llm_messages(
    messages: Vec<ChatRsMessage>,
    user_id: &Uuid,
    session_id: &Uuid,
    db: &mut DbConnection,
    storage: &LocalStorage,
) -> Result<Vec<LlmMessage>, ApiError> {
    let mut llm_messages = Vec::with_capacity(messages.len());

    for message in messages {
        match message.role {
            ChatRsMessageRole::User => {
                let mut files: Option<Vec<LlmFileInput>> = None;
                if let Some(file_ids) = message.meta.user.and_then(|u| u.files) {
                    let mut file_db_service = FileDbService::new(db);
                    for file_id in file_ids {
                        let file = file_db_service
                            .find_session_file(user_id, session_id, &file_id)
                            .await?;
                        let (file_type, content) =
                            file.read_to_string(Some(session_id), storage).await?;
                        files.get_or_insert_default().push(LlmFileInput {
                            name: file.path,
                            content_type: file.content_type,
                            file_type,
                            content,
                        });
                    }
                }
                llm_messages.push(LlmMessage::User(LlmUserMessage {
                    text: message.content,
                    files,
                }))
            }
            ChatRsMessageRole::Assistant => {
                llm_messages.push(LlmMessage::Assistant(LlmAssistantMessage {
                    text: message.content,
                    tool_calls: message.meta.assistant.and_then(|a| a.tool_calls),
                }))
            }
            ChatRsMessageRole::System => llm_messages.push(LlmMessage::System(message.content)),
            ChatRsMessageRole::Tool => {
                if let Some(tool_call) = message.meta.tool_call {
                    llm_messages.push(LlmMessage::Tool(LlmToolResult {
                        tool_call_id: tool_call.id,
                        tool_name: tool_call.tool_name,
                        content: message.content,
                    }))
                }
            }
        }
    }

    Ok(llm_messages)
}
