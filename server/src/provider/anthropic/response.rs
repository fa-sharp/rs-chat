use serde::Deserialize;

use crate::{
    db::models::ChatRsToolCall,
    provider::{
        LlmPendingToolCall, LlmStreamChunk, LlmStreamChunkResult, LlmStreamError, LlmTool, LlmUsage,
    },
};

/// Parse chunks from an Anthropic SSE event.
pub fn parse_anthropic_event(
    event: AnthropicStreamEvent,
    tools: &Option<Vec<LlmTool>>,
    tool_calls: &mut Vec<AnthropicStreamToolCall>,
) -> Vec<LlmStreamChunkResult> {
    let mut chunks: Vec<LlmStreamChunkResult> = Vec::new();
    match event {
        AnthropicStreamEvent::MessageStart { message } => {
            if let Some(usage) = message.usage {
                chunks.push(Ok(LlmStreamChunk::Usage(usage.into())));
            }
        }
        AnthropicStreamEvent::ContentBlockStart {
            content_block,
            index,
        } => match content_block {
            AnthropicResponseContentBlock::Text { text } => {
                chunks.push(Ok(LlmStreamChunk::Text(text)));
            }
            AnthropicResponseContentBlock::ToolUse { id, name } => {
                tool_calls.push(AnthropicStreamToolCall {
                    id,
                    index,
                    name,
                    input: String::with_capacity(100),
                });
            }
        },
        AnthropicStreamEvent::ContentBlockDelta { delta, index } => match delta {
            AnthropicDelta::TextDelta { text } => {
                chunks.push(Ok(LlmStreamChunk::Text(text)));
            }
            AnthropicDelta::InputJsonDelta { partial_json } => {
                if let Some(tool_call) = tool_calls.iter_mut().find(|tc| tc.index == index) {
                    tool_call.input.push_str(&partial_json);
                    let chunk = LlmStreamChunk::PendingToolCall(LlmPendingToolCall {
                        index,
                        tool_name: tool_call.name.clone(),
                    });
                    chunks.push(Ok(chunk));
                }
            }
        },
        AnthropicStreamEvent::ContentBlockStop { index } => {
            if let Some(llm_tools) = tools {
                if let Some(tc) = tool_calls
                    .iter()
                    .position(|tc| tc.index == index)
                    .map(|i| tool_calls.swap_remove(i))
                {
                    if let Some(tool_call) = tc.convert(llm_tools) {
                        let chunk = LlmStreamChunk::ToolCalls(vec![tool_call]);
                        chunks.push(Ok(chunk));
                    }
                }
            }
        }
        AnthropicStreamEvent::MessageDelta { usage } => {
            if let Some(usage) = usage {
                chunks.push(Ok(LlmStreamChunk::Usage(usage.into())));
            }
        }
        AnthropicStreamEvent::Error { error } => {
            let error_msg = format!("{}: {}", error.error_type, error.message);
            chunks.push(Err(LlmStreamError::ProviderError(error_msg)));
        }
        _ => {} // Ignore other events (ping, message_stop)
    }
    chunks
}

/// Anthropic API response content block
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicResponseContentBlock {
    Text { text: String },
    ToolUse { id: String, name: String },
}

/// Anthropic API response usage
#[derive(Debug, Deserialize)]
pub struct AnthropicUsage {
    input_tokens: Option<u32>,
    output_tokens: Option<u32>,
}

impl From<AnthropicUsage> for LlmUsage {
    fn from(usage: AnthropicUsage) -> Self {
        LlmUsage {
            input_tokens: usage.input_tokens,
            output_tokens: usage.output_tokens,
            cost: None,
        }
    }
}

/// Anthropic API response
#[derive(Debug, Deserialize)]
pub struct AnthropicResponse {
    pub content: Vec<AnthropicResponseContentBlock>,
    pub usage: Option<AnthropicUsage>,
}

/// Anthropic stream response (message start)
#[derive(Debug, Deserialize)]
pub struct AnthropicStreamResponse {
    // id: String,
    // #[serde(rename = "type")]
    // message_type: String,
    // role: String,
    // content: Vec<AnthropicContentBlock>,
    // model: String,
    // stop_reason: Option<String>,
    // stop_sequence: Option<String>,
    usage: Option<AnthropicUsage>,
}

/// Anthropic streaming event types
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicStreamEvent {
    MessageStart {
        message: AnthropicStreamResponse,
    },
    ContentBlockStart {
        index: usize,
        content_block: AnthropicResponseContentBlock,
    },
    ContentBlockDelta {
        index: usize,
        delta: AnthropicDelta,
    },
    ContentBlockStop {
        index: usize,
    },
    MessageDelta {
        // delta: AnthropicMessageDelta,
        usage: Option<AnthropicUsage>,
    },
    MessageStop,
    Ping,
    Error {
        error: AnthropicError,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicDelta {
    TextDelta { text: String },
    InputJsonDelta { partial_json: String },
}

#[derive(Debug, Deserialize)]
pub struct AnthropicMessageDelta {
    // stop_reason: Option<String>,
    // stop_sequence: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AnthropicError {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
}

/// Helper struct for tracking streaming tool calls
#[derive(Debug)]
pub struct AnthropicStreamToolCall {
    /// Anthropic tool call ID
    id: String,
    /// Index of the tool call in the message
    index: usize,
    /// Name of the tool
    name: String,
    /// Partial input parameters (JSON stringified)
    input: String,
}

impl AnthropicStreamToolCall {
    /// Convert Anthropic tool call format to ChatRsToolCall
    fn convert(self, llm_tools: &[LlmTool]) -> Option<ChatRsToolCall> {
        let input = if self.input.trim().is_empty() {
            "{}"
        } else {
            &self.input
        };
        let parameters = serde_json::from_str(input).ok()?;
        llm_tools
            .iter()
            .find(|tool| tool.name == self.name)
            .map(|tool| ChatRsToolCall {
                id: self.id,
                tool_id: tool.tool_id,
                tool_name: self.name,
                tool_type: tool.tool_type,
                parameters,
            })
    }
}
