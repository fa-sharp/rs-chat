//! Ollama API request structures

use serde::Serialize;

use crate::{
    db::models::ChatRsFileType,
    provider::{LlmMessage, LlmTool},
    tools::ToolParameters,
};

/// Convert LlmMessages to Ollama messages
pub fn build_ollama_messages(messages: &[LlmMessage]) -> Vec<OllamaMessage> {
    messages
        .iter()
        .map(|message| match message {
            LlmMessage::User(user_message) => {
                let images = user_message.files.as_ref().map(|files| {
                    files
                        .iter()
                        .filter_map(|file| match file.file_type {
                            ChatRsFileType::Image => Some(file.content.as_str()),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                });
                OllamaMessage {
                    role: "user",
                    content: &user_message.text,
                    images,
                    ..Default::default()
                }
            }
            LlmMessage::Assistant(assistant_message) => {
                let tool_calls = assistant_message.tool_calls.as_ref().map(|tool_calls| {
                    tool_calls
                        .iter()
                        .map(|tc| OllamaToolCall {
                            function: OllamaFunction {
                                name: &tc.tool_name,
                                arguments: &tc.parameters,
                            },
                        })
                        .collect()
                });
                OllamaMessage {
                    role: "assistant",
                    content: &assistant_message.text,
                    tool_calls,
                    ..Default::default()
                }
            }
            LlmMessage::System(text) => OllamaMessage {
                role: "system",
                content: text,
                ..Default::default()
            },
            LlmMessage::Tool(result) => OllamaMessage {
                role: "tool",
                content: &result.content,
                tool_name: Some(&result.tool_name),
                ..Default::default()
            },
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
#[derive(Debug, Default, Serialize)]
pub struct OllamaMessage<'a> {
    pub role: &'a str,
    pub content: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<&'a str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OllamaToolCall<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<&'a str>,
}

/// Ollama tool call in a message
#[derive(Debug, Serialize)]
pub struct OllamaToolCall<'a> {
    pub function: OllamaFunction<'a>,
}

/// Ollama tool function
#[derive(Debug, Serialize)]
pub struct OllamaFunction<'a> {
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
