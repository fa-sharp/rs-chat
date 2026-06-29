use futures::{future::BoxFuture, stream::BoxStream};

use super::{
    error::{LlmRequestError, LlmStreamError},
    types::{LlmChatRequest, LlmUsage},
};

/// Trait that all LLM providers must implement
pub trait LlmProvider {
    fn stream_chat<'r>(&'r self, request: &'r LlmChatRequest) -> LlmStreamingResponse<'r>;
}

pub type LlmStreamingResponse<'r> = BoxFuture<'r, Result<LlmStream, LlmRequestError>>;
pub type LlmStream = BoxStream<'static, LlmStreamChunkResult>;
pub type LlmStreamChunkResult = Result<LlmStreamChunk, LlmStreamError>;

/// A streaming chunk of data from the LLM provider
pub enum LlmStreamChunk {
    Text(String),
    Usage(LlmUsage),
    // ToolCalls(Vec<ChatRsToolCall>),
    // PendingToolCall(LlmPendingToolCall),
    // Images(Vec<LlmImage>),
}
