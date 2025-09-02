use std::collections::HashMap;

use serde::Serialize;

use crate::{
    db::models::ChatRsFileType,
    provider::{LlmMessage, LlmTool},
};

pub fn build_anthropic_messages<'a>(
    messages: &'a [LlmMessage],
) -> (Vec<AnthropicMessage<'a>>, Option<&'a str>) {
    let system_prompt = messages.iter().rev().find_map(|message| {
        let LlmMessage::System(msg) = message else {
            return None;
        };
        Some(msg.as_str())
    });

    let anthropic_messages: Vec<AnthropicMessage> = messages
        .iter()
        .filter_map(|message| {
            let mut content_blocks = Vec::new();
            match message {
                LlmMessage::User(user_message) => {
                    if !user_message.text.is_empty() {
                        content_blocks.push(AnthropicContentBlock::Text {
                            text: &user_message.text,
                        });
                    }
                    if let Some(ref files) = user_message.files {
                        content_blocks.extend(files.iter().map(|file| match file.file_type {
                            ChatRsFileType::Text => AnthropicContentBlock::Document {
                                title: &file.name,
                                source: AnthropicSource::Text {
                                    data: &file.content,
                                    media_type: "text/plain",
                                },
                            },
                            ChatRsFileType::Image => AnthropicContentBlock::Image {
                                title: &file.name,
                                source: AnthropicSource::Base64 {
                                    data: &file.content,
                                    media_type: &file.content_type,
                                },
                            },
                            ChatRsFileType::Pdf => AnthropicContentBlock::Document {
                                title: &file.name,
                                source: AnthropicSource::Base64 {
                                    data: &file.content,
                                    media_type: "application/pdf",
                                },
                            },
                        }));
                    }
                    Some(AnthropicMessage {
                        role: "user",
                        content: content_blocks,
                    })
                }
                LlmMessage::Assistant(assistant_message) => {
                    if !assistant_message.text.is_empty() {
                        content_blocks.push(AnthropicContentBlock::Text {
                            text: &assistant_message.text,
                        });
                    }
                    if let Some(ref tool_calls) = assistant_message.tool_calls {
                        content_blocks.extend(tool_calls.iter().map(|tc| {
                            AnthropicContentBlock::ToolUse {
                                id: &tc.id,
                                name: &tc.tool_name,
                                input: &tc.parameters,
                            }
                        }));
                    }
                    Some(AnthropicMessage {
                        role: "assistant",
                        content: content_blocks,
                    })
                }
                LlmMessage::Tool(result) => {
                    content_blocks.push(AnthropicContentBlock::ToolResult {
                        tool_use_id: &result.tool_call_id,
                        content: &result.content,
                    });
                    Some(AnthropicMessage {
                        role: "user",
                        content: content_blocks,
                    })
                }
                _ => None,
            }
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
    Image {
        title: &'a str,
        source: AnthropicSource<'a>,
    },
    Document {
        title: &'a str,
        source: AnthropicSource<'a>,
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

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AnthropicSource<'a> {
    Base64 { data: &'a str, media_type: &'a str },
    Text { data: &'a str, media_type: &'a str },
}
