use futures::{future::BoxFuture, stream::BoxStream};

use super::{
    error::{LlmRequestError, LlmStreamChunkError},
    types::{LlmChatRequest, LlmPrompt, LlmUsage},
};

/// Trait representing an LLM provider
pub trait LlmProvider: Send + Sync {
    fn prompt<'r>(&'r self, prompt: LlmPrompt<'r>) -> LlmPromptResponse<'r>;
    fn stream_chat<'r>(&'r self, request: LlmChatRequest<'r>) -> LlmStreamingResponse<'r>;
}

/// API response to a prompt request from the LLM provider
pub type LlmPromptResponse<'r> = BoxFuture<'r, Result<LlmResponse, LlmRequestError>>;
/// Initial API response to a streaming request from the LLM provider
pub type LlmStreamingResponse<'r> =
    BoxFuture<'r, Result<(LlmStream, LlmResponseMeta), LlmRequestError>>;
/// The response stream from the LLM provider
pub type LlmStream = BoxStream<'static, LlmStreamChunkResult>;
/// The type of the chunks in the LLM response stream
pub type LlmStreamChunkResult = Result<LlmStreamChunk, LlmStreamChunkError>;

/// Prompt response data from the LLM provider
#[derive(Debug, Default)]
pub struct LlmResponse {
    pub text: String,
    pub usage: LlmUsage,
    pub meta: LlmResponseMeta,
}

#[derive(Debug, Default)]
pub struct LlmResponseMeta {
    pub request_id: Option<String>,
}
impl LlmResponseMeta {
    pub fn new(request_id: Option<String>) -> Self {
        Self { request_id }
    }
}

/// A streaming chunk of data from the LLM provider
pub enum LlmStreamChunk {
    Text(String),
    Usage(LlmUsage),
    // ToolCalls(Vec<ChatRsToolCall>),
    // PendingToolCall(LlmPendingToolCall),
    // Images(Vec<LlmImage>),
}
