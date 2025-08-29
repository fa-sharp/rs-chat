use serde::Serialize;

use crate::{
    db::models::{ChatRsMessage, ChatRsMessageRole},
    provider::LlmTool,
};

pub fn build_openai_messages<'a>(messages: &'a [ChatRsMessage]) -> Vec<OpenAIMessage<'a>> {
    messages
        .iter()
        .map(|message| {
            let role = match message.role {
                ChatRsMessageRole::User => "user",
                ChatRsMessageRole::Assistant => "assistant",
                ChatRsMessageRole::System => "system",
                ChatRsMessageRole::Tool => "tool",
            };
            let openai_message = OpenAIMessage {
                role,
                content: Some(&message.content),
                tool_call_id: message.meta.tool_call.as_ref().map(|tc| tc.id.as_str()),
                tool_calls: message
                    .meta
                    .assistant
                    .as_ref()
                    .and_then(|meta| meta.tool_calls.as_ref())
                    .map(|tc| {
                        tc.iter()
                            .map(|tc| OpenAIToolCall {
                                id: &tc.id,
                                tool_type: "function",
                                function: OpenAIToolCallFunction {
                                    name: &tc.tool_name,
                                    arguments: serde_json::to_string(&tc.parameters)
                                        .unwrap_or_default(),
                                },
                            })
                            .collect()
                    }),
            };

            openai_message
        })
        .collect()
}

pub fn build_openai_tools<'a>(tools: &'a [LlmTool]) -> Vec<OpenAITool<'a>> {
    tools
        .iter()
        .map(|tool| OpenAITool {
            tool_type: "function",
            function: OpenAIToolFunction {
                name: &tool.name,
                description: &tool.description,
                parameters: &tool.input_schema,
                strict: true,
            },
        })
        .collect()
}

/// OpenAI API request body
#[derive(Debug, Default, Serialize)]
pub struct OpenAIRequest<'a> {
    pub model: &'a str,
    pub messages: Vec<OpenAIMessage<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u32>,
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<OpenAIStreamOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<OpenAITool<'a>>>,
}

/// OpenAI API request stream options
#[derive(Debug, Serialize)]
pub struct OpenAIStreamOptions {
    pub include_usage: bool,
}

/// OpenAI API request message
#[derive(Debug, Default, Serialize)]
pub struct OpenAIMessage<'a> {
    pub role: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OpenAIToolCall<'a>>>,
}

/// OpenAI tool definition
#[derive(Debug, Serialize)]
pub struct OpenAITool<'a> {
    #[serde(rename = "type")]
    tool_type: &'a str,
    function: OpenAIToolFunction<'a>,
}

/// OpenAI tool function definition
#[derive(Debug, Serialize)]
pub struct OpenAIToolFunction<'a> {
    name: &'a str,
    description: &'a str,
    parameters: &'a serde_json::Value,
    strict: bool,
}

/// OpenAI tool call in messages
#[derive(Debug, Serialize)]
pub struct OpenAIToolCall<'a> {
    id: &'a str,
    #[serde(rename = "type")]
    tool_type: &'a str,
    function: OpenAIToolCallFunction<'a>,
}

/// OpenAI tool call function in messages
#[derive(Debug, Serialize)]
pub struct OpenAIToolCallFunction<'a> {
    name: &'a str,
    arguments: String,
}
