//! Ollama API response structures

use serde::Deserialize;

use crate::llm::{interface::LlmStreamChunk, types::LlmUsage};

/// Parse Ollama streaming event into LlmStreamChunks
pub fn parse_ollama_event(
    event: OllamaStreamEvent,
    // tool_calls: &mut Vec<OllamaToolCallResponse>,
) -> impl Iterator<Item = LlmStreamChunk> {
    // Handle usage stats
    let usage = event.usage().map(LlmStreamChunk::Usage);

    // Handle text response
    let text = (!event.message.content.is_empty())
        .then_some(event.message.content)
        .map(LlmStreamChunk::Text);

    [text, usage].into_iter().flatten()

    // Handle tool calls in the message
    // if !event.message.tool_calls.is_empty() {
    //     for (index, tc) in event.message.tool_calls.iter().enumerate() {
    //         let tool_call = LlmPendingToolCall {
    //             index,
    //             tool_name: tc.function.name.clone(),
    //         };
    //         chunks.push(Ok(LlmStreamChunk::PendingToolCall(tool_call)));
    //     }
    //     tool_calls.extend(event.message.tool_calls);
    // }
}

/// Ollama chat response (streaming)
#[derive(Debug, Deserialize)]
pub struct OllamaStreamEvent {
    // pub model: String,
    // pub created_at: String,
    pub message: OllamaMessageResponse,
    pub done: bool,
    // #[serde(default)]
    // pub done_reason: Option<String>,
    // #[serde(default)]
    // pub total_duration: Option<u64>,
    // #[serde(default)]
    // pub load_duration: Option<u64>,
    #[serde(default)]
    pub prompt_eval_count: Option<i32>,
    // #[serde(default)]
    // pub prompt_eval_duration: Option<u64>,
    #[serde(default)]
    pub eval_count: Option<i32>,
    // #[serde(default)]
    // pub eval_duration: Option<u64>,
}

/// Ollama completion response (non-streaming)
#[derive(Debug, Deserialize)]
pub struct OllamaCompletionResponse {
    pub response: String,
    // pub model: String,
    // pub created_at: String,
    // pub done: bool,
    // #[serde(default)]
    // pub done_reason: Option<String>,
    // #[serde(default)]
    // pub total_duration: Option<u64>,
    // #[serde(default)]
    // pub load_duration: Option<u64>,
    // #[serde(default)]
    // pub prompt_eval_duration: Option<u64>,
    // #[serde(default)]
    // pub eval_duration: Option<u64>,
    #[serde(default)]
    pub prompt_eval_count: Option<i32>,
    #[serde(default)]
    pub eval_count: Option<i32>,
}

/// Ollama message in response
#[derive(Debug, Deserialize)]
pub struct OllamaMessageResponse {
    #[serde(default)]
    pub content: String,
    // pub role: String,
    // #[serde(default)]
    // pub tool_calls: Vec<OllamaToolCallResponse>,
}

// /// Ollama tool call in response
// #[derive(Debug, Deserialize)]
// pub struct OllamaToolCallResponse {
//     pub function: OllamaFunctionResponse,
// }

// /// Ollama tool function in response
// #[derive(Debug, Deserialize)]
// pub struct OllamaFunctionResponse {
//     pub name: String,
//     pub arguments: serde_json::Value,
// }

// impl OllamaFunctionResponse {
//     /// Convert to ChatRsToolCall if the tool exists in the provided tools
//     pub fn convert(self, tools: &[LlmTool]) -> Option<ChatRsToolCall> {
//         let tool = tools.iter().find(|t| t.name == self.name)?;
//         let parameters = serde_json::from_value(self.arguments).ok()?;

//         Some(ChatRsToolCall {
//             id: uuid::Uuid::new_v4().to_string(),
//             parameters,
//             tool_id: tool.tool_id,
//             tool_name: self.name,
//             tool_type: tool.tool_type,
//         })
//     }
// }

impl OllamaCompletionResponse {
    /// Convert usage to LlmUsage
    pub fn usage(&self) -> Option<LlmUsage> {
        if self.prompt_eval_count.is_some() || self.eval_count.is_some() {
            Some(LlmUsage {
                input_tokens: self.prompt_eval_count,
                output_tokens: self.eval_count,
                ..Default::default()
            })
        } else {
            None
        }
    }
}

impl OllamaStreamEvent {
    /// If last event in stream, convert usage to LlmUsage
    pub fn usage(&self) -> Option<LlmUsage> {
        if self.done && (self.prompt_eval_count.is_some() || self.eval_count.is_some()) {
            Some(LlmUsage {
                input_tokens: self.prompt_eval_count,
                output_tokens: self.eval_count,
                ..Default::default()
            })
        } else {
            None
        }
    }
}
