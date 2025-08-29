//! Ollama API response structures

use serde::Deserialize;

use crate::{
    db::models::ChatRsToolCall,
    provider::{LlmPendingToolCall, LlmStreamChunk, LlmStreamError, LlmTool, LlmUsage},
};

/// Parse Ollama streaming event into LlmStreamChunks, and track tool calls
pub fn parse_ollama_event(
    event: OllamaStreamResponse,
    tool_calls: &mut Vec<OllamaToolCall>,
) -> Vec<Result<LlmStreamChunk, LlmStreamError>> {
    let mut chunks = Vec::with_capacity(1);
    // Handle final message with usage stats
    if event.done {
        if let Some(usage) = Option::<LlmUsage>::from(&event) {
            chunks.push(Ok(LlmStreamChunk::Usage(usage)));
        }
    }

    // Handle tool calls in the message
    if !event.message.tool_calls.is_empty() {
        for (index, tc) in event.message.tool_calls.iter().enumerate() {
            let tool_call = LlmPendingToolCall {
                index,
                tool_name: tc.function.name.clone(),
            };
            chunks.push(Ok(LlmStreamChunk::PendingToolCall(tool_call)));
        }
        tool_calls.extend(event.message.tool_calls);
    }

    // Handle text content
    if !event.message.content.is_empty() {
        chunks.push(Ok(LlmStreamChunk::Text(event.message.content)));
    }

    chunks
}

/// Ollama chat response (streaming)
#[derive(Debug, Deserialize)]
pub struct OllamaStreamResponse {
    pub model: String,
    pub created_at: String,
    pub message: OllamaMessage,
    pub done: bool,
    #[serde(default)]
    pub done_reason: Option<String>,
    #[serde(default)]
    pub total_duration: Option<u64>,
    #[serde(default)]
    pub load_duration: Option<u64>,
    #[serde(default)]
    pub prompt_eval_count: Option<u32>,
    #[serde(default)]
    pub prompt_eval_duration: Option<u64>,
    #[serde(default)]
    pub eval_count: Option<u32>,
    #[serde(default)]
    pub eval_duration: Option<u64>,
}

/// Ollama completion response (non-streaming)
#[derive(Debug, Deserialize)]
pub struct OllamaCompletionResponse {
    pub model: String,
    pub created_at: String,
    pub response: String,
    pub done: bool,
    #[serde(default)]
    pub done_reason: Option<String>,
    #[serde(default)]
    pub total_duration: Option<u64>,
    #[serde(default)]
    pub load_duration: Option<u64>,
    #[serde(default)]
    pub prompt_eval_count: Option<u32>,
    #[serde(default)]
    pub prompt_eval_duration: Option<u64>,
    #[serde(default)]
    pub eval_count: Option<u32>,
    #[serde(default)]
    pub eval_duration: Option<u64>,
}

/// Ollama message in response
#[derive(Debug, Deserialize)]
pub struct OllamaMessage {
    pub role: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub tool_calls: Vec<OllamaToolCall>,
}

/// Ollama tool call in response
#[derive(Debug, Deserialize)]
pub struct OllamaToolCall {
    pub function: OllamaToolFunction,
}

/// Ollama tool function in response
#[derive(Debug, Deserialize)]
pub struct OllamaToolFunction {
    pub name: String,
    pub arguments: serde_json::Value,
}

impl OllamaToolFunction {
    /// Convert to ChatRsToolCall if the tool exists in the provided tools
    pub fn convert(self, tools: &[LlmTool]) -> Option<ChatRsToolCall> {
        let tool = tools.iter().find(|t| t.name == self.name)?;
        let parameters = serde_json::from_value(self.arguments).ok()?;

        Some(ChatRsToolCall {
            id: uuid::Uuid::new_v4().to_string(),
            parameters,
            tool_id: tool.tool_id,
            tool_name: self.name,
            tool_type: tool.tool_type,
        })
    }
}

/// Convert Ollama usage to LlmUsage
impl From<&OllamaCompletionResponse> for Option<LlmUsage> {
    fn from(response: &OllamaCompletionResponse) -> Self {
        if response.prompt_eval_count.is_some() || response.eval_count.is_some() {
            Some(LlmUsage {
                input_tokens: response.prompt_eval_count,
                output_tokens: response.eval_count,
                cost: None,
            })
        } else {
            None
        }
    }
}

impl From<&OllamaStreamResponse> for Option<LlmUsage> {
    fn from(response: &OllamaStreamResponse) -> Self {
        if response.prompt_eval_count.is_some() || response.eval_count.is_some() {
            Some(LlmUsage {
                input_tokens: response.prompt_eval_count,
                output_tokens: response.eval_count,
                cost: None,
            })
        } else {
            None
        }
    }
}

/// Ollama models list response
#[derive(Debug, Deserialize)]
pub struct OllamaModelsResponse {
    pub models: Vec<OllamaModelInfo>,
}

/// Ollama model information
#[derive(Debug, Deserialize)]
pub struct OllamaModelInfo {
    pub name: String,
    // pub model: String,
    pub modified_at: String,
    // pub size: u64,
    // pub digest: String,
    pub details: OllamaModelDetails,
}

/// Ollama model details
#[derive(Debug, Deserialize)]
pub struct OllamaModelDetails {
    // #[serde(default)]
    // pub parent_model: String,
    pub format: String,
    pub family: String,
    // #[serde(default)]
    // pub families: Vec<String>,
    // pub parameter_size: String,
    // #[serde(default)]
    // pub quantization_level: Option<String>,
}
