use std::collections::HashMap;

use serde::Serialize;

use crate::{
    db::models::{ChatRsMessage, ChatRsMessageRole},
    provider::LlmTool,
};

pub fn build_anthropic_messages<'a>(
    messages: &'a [ChatRsMessage],
) -> (Vec<AnthropicMessage<'a>>, Option<&'a str>) {
    let system_prompt = messages
        .iter()
        .rfind(|message| message.role == ChatRsMessageRole::System)
        .map(|message| message.content.as_str());

    let anthropic_messages: Vec<AnthropicMessage> = messages
        .iter()
        .filter_map(|message| {
            let role = match message.role {
                ChatRsMessageRole::User => "user",
                ChatRsMessageRole::Tool => "user",
                ChatRsMessageRole::Assistant => "assistant",
                ChatRsMessageRole::System => return None,
            };

            let mut content_blocks = Vec::new();

            // Handle tool result messages
            if message.role == ChatRsMessageRole::Tool {
                if let Some(executed_call) = &message.meta.tool_call {
                    content_blocks.push(AnthropicContentBlock::ToolResult {
                        tool_use_id: &executed_call.id,
                        content: &message.content,
                    });
                }
            } else {
                // Handle regular text content
                if !message.content.is_empty() {
                    content_blocks.push(AnthropicContentBlock::Text {
                        text: &message.content,
                    });
                }
                // Handle tool calls in assistant messages
                if let Some(tool_calls) = message
                    .meta
                    .assistant
                    .as_ref()
                    .and_then(|a| a.tool_calls.as_ref())
                {
                    for tool_call in tool_calls {
                        content_blocks.push(AnthropicContentBlock::ToolUse {
                            id: &tool_call.id,
                            name: &tool_call.tool_name,
                            input: &tool_call.parameters,
                        });
                    }
                }
            }

            if content_blocks.is_empty() {
                return None;
            }

            Some(AnthropicMessage {
                role,
                content: content_blocks,
            })
        })
        .collect();

    (anthropic_messages, system_prompt)
}

pub fn build_anthropic_tools<'a>(tools: &'a [LlmTool]) -> Vec<AnthropicTool<'a>> {
    tools
        .iter()
        .map(|tool| AnthropicTool {
            name: &tool.name,
            description: &tool.description,
            input_schema: &tool.input_schema,
        })
        .collect()
}

/// Anthropic API request message
#[derive(Debug, Serialize)]
pub struct AnthropicMessage<'a> {
    pub role: &'a str,
    pub content: Vec<AnthropicContentBlock<'a>>,
}

/// Anthropic API request body
#[derive(Debug, Serialize)]
pub struct AnthropicRequest<'a> {
    pub model: &'a str,
    pub messages: Vec<AnthropicMessage<'a>>,
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<AnthropicTool<'a>>>,
}

/// Anthropic tool definition
#[derive(Debug, Serialize)]
pub struct AnthropicTool<'a> {
    name: &'a str,
    description: &'a str,
    input_schema: &'a serde_json::Value,
}

/// Anthropic content block for messages
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicContentBlock<'a> {
    Text {
        text: &'a str,
    },
    ToolUse {
        id: &'a str,
        name: &'a str,
        input: &'a HashMap<String, serde_json::Value>,
    },
    ToolResult {
        tool_use_id: &'a str,
        content: &'a str,
    },
}
