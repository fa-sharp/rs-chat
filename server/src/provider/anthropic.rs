//! Anthropic LLM provider

use std::collections::HashMap;

use rocket::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    db::models::{ChatRsMessage, ChatRsMessageRole, ChatRsTool, ChatRsToolCall},
    provider::{
        LlmApiProvider, LlmApiProviderSharedOptions, LlmApiStream, LlmError, LlmStreamChunk,
        LlmUsage, DEFAULT_MAX_TOKENS,
    },
    provider_models::{LlmModel, ModelsDevService, ModelsDevServiceProvider},
};

const MESSAGES_API_URL: &str = "https://api.anthropic.com/v1/messages";
const API_VERSION: &str = "2023-06-01";

/// Anthropic chat provider
pub struct AnthropicProvider<'a> {
    client: reqwest::Client,
    redis: &'a fred::prelude::Client,
    api_key: &'a str,
}

impl<'a> AnthropicProvider<'a> {
    pub fn new(
        http_client: &reqwest::Client,
        redis: &'a fred::prelude::Client,
        api_key: &'a str,
    ) -> Self {
        Self {
            client: http_client.clone(),
            redis,
            api_key,
        }
    }

    fn build_messages(
        &self,
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
                    if let Some(executed_call) = &message.meta.executed_tool_call {
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
                    if let Some(tool_calls) = &message.meta.tool_calls {
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

    fn build_tools(&self, tools: &'a [ChatRsTool]) -> Vec<AnthropicTool<'a>> {
        tools
            .iter()
            .map(|tool| AnthropicTool {
                name: &tool.name,
                description: &tool.description,
                input_schema: tool.get_input_schema(),
            })
            .collect()
    }

    async fn parse_sse_stream(
        &self,
        mut response: reqwest::Response,
        tools: Option<Vec<ChatRsTool>>,
    ) -> LlmApiStream {
        let stream = async_stream::stream! {
            let mut buffer = String::new();
            let mut current_tool_calls: Vec<Option<AnthropicStreamToolCall>> = Vec::new();

            while let Some(chunk) = response.chunk().await.transpose() {
                match chunk {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        buffer.push_str(&text);

                        while let Some(line_end_idx) = buffer.find('\n') {
                            let line = buffer[..line_end_idx].trim_end_matches('\r');

                            if line.starts_with("data: ") {
                                let data = &line[6..]; // Remove "data: " prefix
                                if data.trim().is_empty() || data == "[DONE]" {
                                    buffer.drain(..=line_end_idx);
                                    continue;
                                }

                                match serde_json::from_str::<AnthropicStreamEvent>(data) {
                                    Ok(event) => {
                                        match event {
                                            AnthropicStreamEvent::MessageStart { message } => {
                                                if let Some(usage) = message.usage {
                                                    yield Ok(LlmStreamChunk {
                                                        text: Some(String::new()),
                                                        tool_calls: None,
                                                        usage: Some(usage.into()),
                                                    });
                                                }
                                            }
                                            AnthropicStreamEvent::ContentBlockStart { content_block, index } => {
                                                match content_block {
                                                    AnthropicResponseContentBlock::Text { text } => {
                                                        yield Ok(LlmStreamChunk {
                                                            text: Some(text),
                                                            tool_calls: None,
                                                            usage: None,
                                                        });
                                                    }
                                                    AnthropicResponseContentBlock::ToolUse { id,  name } => {
                                                        current_tool_calls.push(Some(AnthropicStreamToolCall {
                                                            id,
                                                            index,
                                                            name,
                                                            input: String::with_capacity(100),
                                                        }));
                                                    }
                                                }
                                            }
                                            AnthropicStreamEvent::ContentBlockDelta { delta, index } => {
                                                match delta {
                                                    AnthropicDelta::TextDelta { text } => {
                                                        yield Ok(LlmStreamChunk {
                                                            text: Some(text),
                                                            tool_calls: None,
                                                            usage: None,
                                                        });
                                                    }
                                                    AnthropicDelta::InputJsonDelta { partial_json } => {
                                                        if let Some(Some(tool_call)) = current_tool_calls.iter_mut().find(|tc| tc.as_ref().is_some_and(|tc| tc.index == index)) {
                                                            tool_call.input.push_str(&partial_json);
                                                        }
                                                    }
                                                }
                                            }
                                            AnthropicStreamEvent::ContentBlockStop { index } => {
                                                if let Some(rs_chat_tools) = &tools {
                                                    if !current_tool_calls.is_empty() {
                                                        let converted_call = current_tool_calls
                                                            .iter_mut()
                                                            .find(|tc| tc.as_ref().is_some_and(|tc| tc.index == index))
                                                            .and_then(|tc| tc.take())
                                                            .and_then(|tc| tc.convert(rs_chat_tools));
                                                        if let Some(converted_call) = converted_call {
                                                            yield Ok(LlmStreamChunk {
                                                                text: None,
                                                                tool_calls: Some(vec![converted_call]),
                                                                usage: None,
                                                            });
                                                        }
                                                    }
                                                }
                                            }
                                            AnthropicStreamEvent::MessageDelta { usage } => {
                                                if let Some(usage) = usage {
                                                    yield Ok(LlmStreamChunk {
                                                        text: Some(String::new()),
                                                        tool_calls: None,
                                                        usage: Some(usage.into()),
                                                    });
                                                }
                                            }
                                            AnthropicStreamEvent::Error { error } => {
                                                yield Err(LlmError::AnthropicError(
                                                    format!("{}: {}", error.error_type, error.message)
                                                ));
                                            }
                                            _ => {} // Ignore other events (ping, message_stop)
                                        }
                                    }
                                    Err(e) => {
                                        rocket::warn!("Failed to parse SSE event: {} | Data: {}", e, data);
                                    }
                                }
                            } else if line.starts_with("event: ") {
                                let event_type = &line[7..];
                                rocket::debug!("SSE event type: {}", event_type);
                            } else if !line.trim().is_empty() && !line.starts_with(":") {
                                rocket::debug!("Unexpected SSE line: {}", line);
                            }

                            buffer.drain(..=line_end_idx);
                        }
                    }
                    Err(e) => {
                        rocket::warn!("Stream chunk error: {}", e);
                        yield Err(LlmError::AnthropicError(format!("Stream error: {}", e)));
                        break;
                    }
                }
            }

            rocket::debug!("Anthropic SSE stream ended");
        };

        Box::pin(stream)
    }
}

#[async_trait]
impl<'a> LlmApiProvider for AnthropicProvider<'a> {
    async fn chat_stream(
        &self,
        messages: Vec<ChatRsMessage>,
        tools: Option<Vec<ChatRsTool>>,
        options: &LlmApiProviderSharedOptions,
    ) -> Result<LlmApiStream, LlmError> {
        let (anthropic_messages, system_prompt) = self.build_messages(&messages);
        let anthropic_tools = tools.as_ref().map(|t| self.build_tools(t));

        let request = AnthropicRequest {
            model: &options.model,
            messages: anthropic_messages,
            max_tokens: options.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS),
            temperature: options.temperature,
            system: system_prompt,
            stream: Some(true),
            tools: anthropic_tools,
        };

        let response = self
            .client
            .post(MESSAGES_API_URL)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
            .header("x-api-key", self.api_key)
            .json(&request)
            .send()
            .await
            .map_err(|e| LlmError::AnthropicError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(LlmError::AnthropicError(format!(
                "API error {}: {}",
                status, error_text
            )));
        }

        Ok(self.parse_sse_stream(response, tools).await)
    }

    async fn prompt(
        &self,
        message: &str,
        options: &LlmApiProviderSharedOptions,
    ) -> Result<String, LlmError> {
        let request = AnthropicRequest {
            model: &options.model,
            messages: vec![AnthropicMessage {
                role: "user",
                content: vec![AnthropicContentBlock::Text { text: message }],
            }],
            max_tokens: options.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS),
            temperature: options.temperature,
            system: None,
            stream: None,
            tools: None,
        };

        let response = self
            .client
            .post(MESSAGES_API_URL)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
            .header("x-api-key", self.api_key)
            .json(&request)
            .send()
            .await
            .map_err(|e| LlmError::AnthropicError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(LlmError::AnthropicError(format!(
                "API error {}: {}",
                status, error_text
            )));
        }

        let anthropic_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| LlmError::AnthropicError(format!("Failed to parse response: {}", e)))?;

        let text = anthropic_response
            .content
            .first()
            .and_then(|block| match block {
                AnthropicResponseContentBlock::Text { text } => Some(text.clone()),
                _ => None,
            })
            .ok_or_else(|| LlmError::NoResponse)?;

        if let Some(usage) = anthropic_response.usage {
            let usage: LlmUsage = usage.into();
            println!("Prompt usage: {:?}", usage);
        }

        Ok(text)
    }

    async fn list_models(&self) -> Result<Vec<LlmModel>, LlmError> {
        let models_service = ModelsDevService::new(self.redis.clone(), self.client.clone());
        let models = models_service
            .list_models(ModelsDevServiceProvider::Anthropic)
            .await?;

        Ok(models)
    }
}

/// Anthropic API request message
#[derive(Debug, Serialize)]
struct AnthropicMessage<'a> {
    role: &'a str,
    content: Vec<AnthropicContentBlock<'a>>,
}

/// Anthropic API request body
#[derive(Debug, Serialize)]
struct AnthropicRequest<'a> {
    model: &'a str,
    messages: Vec<AnthropicMessage<'a>>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<AnthropicTool<'a>>>,
}

/// Anthropic tool definition
#[derive(Debug, Serialize)]
struct AnthropicTool<'a> {
    name: &'a str,
    description: &'a str,
    input_schema: serde_json::Value,
}

/// Anthropic content block for messages
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AnthropicContentBlock<'a> {
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

/// Anthropic API response content block
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AnthropicResponseContentBlock {
    Text { text: String },
    ToolUse { id: String, name: String },
}

/// Anthropic API response usage
#[derive(Debug, Deserialize)]
struct AnthropicUsage {
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
struct AnthropicResponse {
    content: Vec<AnthropicResponseContentBlock>,
    usage: Option<AnthropicUsage>,
}

/// Anthropic stream response (message start)
#[derive(Debug, Deserialize)]
struct AnthropicStreamResponse {
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
enum AnthropicStreamEvent {
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
enum AnthropicDelta {
    TextDelta { text: String },
    InputJsonDelta { partial_json: String },
}

#[derive(Debug, Deserialize)]
struct AnthropicMessageDelta {
    // stop_reason: Option<String>,
    // stop_sequence: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicError {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
}

/// Helper struct for tracking streaming tool calls
#[derive(Debug)]
struct AnthropicStreamToolCall {
    id: String,
    index: usize,
    name: String,
    /// Partial input parameters (JSON stringified)
    input: String,
}

impl AnthropicStreamToolCall {
    /// Convert Anthropic tool call format to ChatRsToolCall
    fn convert(self, rs_chat_tools: &[ChatRsTool]) -> Option<ChatRsToolCall> {
        let parameters = serde_json::from_str(&self.input).ok()?;
        rs_chat_tools
            .iter()
            .find(|tool| tool.name == self.name)
            .map(|tool| ChatRsToolCall {
                id: self.id,
                tool_id: tool.id,
                tool_name: self.name,
                parameters,
            })
    }
}
