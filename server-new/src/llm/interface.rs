use futures::{future::BoxFuture, stream::BoxStream};

use super::{
    error::{LlmRequestError, LlmStreamChunkError},
    types::{LlmChatRequest, LlmUsage},
};

/// Trait for all LLM providers
pub trait LlmProvider: Send + Sync {
    fn stream_chat<'r>(&'r self, request: LlmChatRequest<'r>) -> LlmStreamingResponse<'r>;
}

/// Initial API response to a streaming request from the LLM provider
pub type LlmStreamingResponse<'r> = BoxFuture<'r, Result<LlmStream, LlmRequestError>>;
/// The response stream from the LLM provider
pub type LlmStream = BoxStream<'static, LlmStreamChunkResult>;
/// The type of the chunks in the LLM response stream
pub type LlmStreamChunkResult = Result<LlmStreamChunk, LlmStreamChunkError>;

/// A streaming chunk of data from the LLM provider
pub enum LlmStreamChunk {
    Text(String),
    Usage(LlmUsage),
    // ToolCalls(Vec<ChatRsToolCall>),
    // PendingToolCall(LlmPendingToolCall),
    // Images(Vec<LlmImage>),
}
