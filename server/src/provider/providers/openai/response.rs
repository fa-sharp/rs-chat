use serde::Deserialize;

use crate::{
    db::models::ChatRsToolCall,
    provider::{LlmPendingToolCall, LlmStreamChunk, LlmStreamChunkResult, LlmTool, LlmUsage},
};

/// Parse chunks from an OpenAI SSE event
pub fn parse_openai_event(
    mut event: OpenAIStreamResponse,
    tool_calls: &mut Vec<OpenAIStreamToolCall>,
) -> Vec<LlmStreamChunkResult> {
    let mut chunks = Vec::with_capacity(1);
    if let Some(delta) = event.choices.pop().and_then(|c| c.delta) {
        if let Some(text) = delta.content {
            chunks.push(Ok(LlmStreamChunk::Text(text)));
        }
        if let Some(tool_calls_delta) = delta.tool_calls {
            for tool_call_delta in tool_calls_delta {
                if let Some(tc) = tool_calls
                    .iter_mut()
                    .find(|tc| tc.index == tool_call_delta.index)
                {
                    if let Some(function_arguments) = tool_call_delta.function.arguments {
                        *tc.function.arguments.get_or_insert_default() += &function_arguments;
                    }
                    if let Some(ref tool_name) = tc.function.name {
                        let chunk = LlmStreamChunk::PendingToolCall(LlmPendingToolCall {
                            index: tool_call_delta.index,
                            tool_name: tool_name.clone(),
                        });
                        chunks.push(Ok(chunk));
                    }
                } else {
                    if let Some(ref tool_name) = tool_call_delta.function.name {
                        let chunk = LlmStreamChunk::PendingToolCall(LlmPendingToolCall {
                            index: tool_call_delta.index,
                            tool_name: tool_name.clone(),
                        });
                        chunks.push(Ok(chunk));
                    }
                    tool_calls.push(tool_call_delta);
                }
            }
        }
    }
    if let Some(usage) = event.usage {
        chunks.push(Ok(LlmStreamChunk::Usage(usage.into())));
    }

    chunks
}

/// OpenAI API response
#[derive(Debug, Deserialize)]
pub struct OpenAIResponse {
    pub choices: Vec<OpenAIChoice>,
    pub usage: Option<OpenAIUsage>,
}

/// OpenAI API streaming response
#[derive(Debug, Deserialize)]
pub struct OpenAIStreamResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

/// OpenAI API response choice
#[derive(Debug, Deserialize)]
pub struct OpenAIChoice {
    pub message: Option<OpenAIResponseMessage>,
    pub delta: Option<OpenAIResponseDelta>,
    // finish_reason: Option<String>,
}

/// OpenAI API response message
#[derive(Debug, Deserialize)]
pub struct OpenAIResponseMessage {
    // role: String,
    pub content: Option<String>,
}

/// OpenAI API streaming delta
#[derive(Debug, Deserialize)]
pub struct OpenAIResponseDelta {
    // role: Option<String>,
    content: Option<String>,
    tool_calls: Option<Vec<OpenAIStreamToolCall>>,
}

/// OpenAI streaming tool call
#[derive(Debug, Deserialize)]
pub struct OpenAIStreamToolCall {
    id: Option<String>,
    index: usize,
    function: OpenAIStreamToolCallFunction,
}

impl OpenAIStreamToolCall {
    /// Convert OpenAI tool call format to ChatRsToolCall, add tool ID
    pub fn convert(self, rs_chat_tools: &[LlmTool]) -> Option<ChatRsToolCall> {
        let id = self.id?;
        let tool_name = self.function.name?;
        let parameters = serde_json::from_str(&self.function.arguments?).ok()?;
        rs_chat_tools
            .iter()
            .find(|tool| tool.name == tool_name)
            .map(|tool| ChatRsToolCall {
                id,
                tool_id: tool.tool_id,
                tool_name,
                tool_type: tool.tool_type,
                parameters,
            })
    }
}

/// OpenAI streaming tool call function
#[derive(Debug, Deserialize)]
struct OpenAIStreamToolCallFunction {
    name: Option<String>,
    arguments: Option<String>,
}

/// OpenAI API response usage
#[derive(Debug, Deserialize)]
pub struct OpenAIUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    cost: Option<f32>,
    // total_tokens: Option<u32>,
}

impl From<OpenAIUsage> for LlmUsage {
    fn from(usage: OpenAIUsage) -> Self {
        LlmUsage {
            input_tokens: usage.prompt_tokens,
            output_tokens: usage.completion_tokens,
            cost: usage.cost,
        }
    }
}
