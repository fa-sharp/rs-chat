//! Ollama API request structures

use serde::Serialize;

use crate::{
    db::models::{ChatRsMessage, ChatRsMessageRole},
    provider::LlmTool,
    tools::ToolParameters,
};

/// Convert ChatRsMessages to Ollama messages
pub fn build_ollama_messages(messages: &[ChatRsMessage]) -> Vec<OllamaMessage> {
    messages
        .iter()
        .map(|msg| {
            let role = match msg.role {
                ChatRsMessageRole::User => "user",
                ChatRsMessageRole::Assistant => "assistant",
                ChatRsMessageRole::System => "system",
                ChatRsMessageRole::Tool => "tool",
            };

            let mut ollama_msg = OllamaMessage {
                role,
                content: &msg.content,
                tool_calls: None,
                tool_name: None,
            };

            // Handle tool calls in assistant messages
            if msg.role == ChatRsMessageRole::Assistant {
                if let Some(msg_tool_calls) = msg
                    .meta
                    .assistant
                    .as_ref()
                    .and_then(|m| m.tool_calls.as_ref())
                {
                    let tool_calls = msg_tool_calls
                        .iter()
                        .map(|tc| OllamaToolCall {
                            function: OllamaToolFunction {
                                name: &tc.tool_name,
                                arguments: &tc.parameters,
                            },
                        })
                        .collect();
                    ollama_msg.tool_calls = Some(tool_calls);
                }
            }

            // Handle tool messages (results from tool calls)
            if msg.role == ChatRsMessageRole::Tool {
                if let Some(ref tool_call) = msg.meta.tool_call {
                    ollama_msg.tool_name = Some(&tool_call.tool_name);
                }
            }

            ollama_msg
        })
        .collect()
}

/// Convert LlmTools to Ollama tools
pub fn build_ollama_tools(tools: &[LlmTool]) -> Vec<OllamaTool> {
    tools
        .iter()
        .map(|tool| OllamaTool {
            r#type: "function",
            function: OllamaToolSpec {
                name: &tool.name,
                description: &tool.description,
                parameters: &tool.input_schema,
            },
        })
        .collect()
}

/// Ollama chat request structure
#[derive(Debug, Serialize)]
pub struct OllamaChatRequest<'a> {
    pub model: &'a str,
    pub messages: Vec<OllamaMessage<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<OllamaTool<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<OllamaOptions>,
}

/// Ollama completion request structure
#[derive(Debug, Serialize)]
pub struct OllamaCompletionRequest<'a> {
    pub model: &'a str,
    pub prompt: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<OllamaOptions>,
}

/// Ollama chat message
#[derive(Debug, Serialize)]
pub struct OllamaMessage<'a> {
    pub role: &'a str,
    pub content: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OllamaToolCall<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<&'a str>,
}

/// Ollama tool call in a message
#[derive(Debug, Serialize)]
pub struct OllamaToolCall<'a> {
    pub function: OllamaToolFunction<'a>,
}

/// Ollama tool function
#[derive(Debug, Serialize)]
pub struct OllamaToolFunction<'a> {
    pub name: &'a str,
    pub arguments: &'a ToolParameters,
}

/// Ollama tool definition
#[derive(Debug, Serialize)]
pub struct OllamaTool<'a> {
    pub r#type: &'a str,
    pub function: OllamaToolSpec<'a>,
}

/// Ollama tool specification
#[derive(Debug, Serialize)]
pub struct OllamaToolSpec<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub parameters: &'a serde_json::Value,
}

/// Ollama model options
#[derive(Debug, Default, Serialize)]
pub struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<u32>, // Ollama's equivalent to max_tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i32>,
}

impl Default for OllamaChatRequest<'_> {
    fn default() -> Self {
        Self {
            model: "",
            messages: Vec::new(),
            tools: None,
            stream: None,
            options: None,
        }
    }
}
