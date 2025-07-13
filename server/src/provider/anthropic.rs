use rocket::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    db::models::{ChatRsMessage, ChatRsMessageRole, ChatRsTool},
    provider::{
        ChatRsError, ChatRsProvider, ChatRsStream, ChatRsStreamChunk, ChatRsUsage,
        DEFAULT_MAX_TOKENS,
    },
};

const MESSAGES_API_URL: &str = "https://api.anthropic.com/v1/messages";
const API_VERSION: &str = "2023-06-01";

/// Anthropic chat provider
pub struct AnthropicProvider<'a> {
    client: reqwest::Client,
    api_key: String,
    model: &'a str,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

impl<'a> AnthropicProvider<'a> {
    pub fn new(
        http_client: &reqwest::Client,
        api_key: &str,
        model: &'a str,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    ) -> Self {
        Self {
            client: http_client.clone(),
            api_key: api_key.to_string(),
            model,
            max_tokens,
            temperature,
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
                    ChatRsMessageRole::Assistant => "assistant",
                    ChatRsMessageRole::System => return None,
                    ChatRsMessageRole::Tool => return None,
                };
                Some(AnthropicMessage {
                    role,
                    content: &message.content,
                })
            })
            .collect();

        (anthropic_messages, system_prompt)
    }

    async fn parse_sse_stream(&self, mut response: reqwest::Response) -> ChatRsStream {
        let stream = async_stream::stream! {
            let mut buffer = String::new();

            while let Some(chunk) = response.chunk().await.transpose() {
                match chunk {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        buffer.push_str(&text);

                        // Process complete lines
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
                                                    yield Ok(ChatRsStreamChunk {
                                                        text: Some(String::new()),
                                                        tool_calls: None,
                                                        usage: Some(usage.into()),
                                                    });
                                                }
                                            }
                                            AnthropicStreamEvent::ContentBlockStart { content_block } => {
                                                if content_block.block_type == "text" {
                                                    yield Ok(ChatRsStreamChunk {
                                                        text: Some(content_block.text),
                                                        tool_calls: None,
                                                        usage: None,
                                                    });
                                                }
                                            }
                                            AnthropicStreamEvent::ContentBlockDelta { delta } => {
                                                match delta {
                                                    AnthropicDelta::TextDelta { text } => {
                                                        yield Ok(ChatRsStreamChunk {
                                                            text: Some(text),
                                                            tool_calls: None,
                                                            usage: None,
                                                        });
                                                    }
                                                    AnthropicDelta::InputJsonDelta { .. } => {
                                                        // TODO: Handle tool use if needed
                                                    }
                                                }
                                            }
                                            AnthropicStreamEvent::MessageDelta { usage } => {
                                                if let Some(usage) = usage {
                                                    yield Ok(ChatRsStreamChunk {
                                                        text: Some(String::new()),
                                                        tool_calls: None,
                                                        usage: Some(usage.into()),
                                                    });
                                                }
                                            }
                                            AnthropicStreamEvent::Error { error } => {
                                                yield Err(ChatRsError::AnthropicError(
                                                    format!("{}: {}", error.error_type, error.message)
                                                ));
                                            }
                                            _ => {} // Ignore other events (ping, content_block_stop, message_stop)
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
                        yield Err(ChatRsError::AnthropicError(format!("Stream error: {}", e)));
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
impl<'a> ChatRsProvider for AnthropicProvider<'a> {
    async fn chat_stream(
        &self,
        messages: Vec<ChatRsMessage>,
        _tools: Option<Vec<ChatRsTool>>,
    ) -> Result<ChatRsStream, ChatRsError> {
        let (anthropic_messages, system_prompt) = self.build_messages(&messages);

        let request = AnthropicRequest {
            model: self.model,
            messages: anthropic_messages,
            max_tokens: self.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS),
            temperature: self.temperature,
            system: system_prompt,
            stream: Some(true),
        };

        let response = self
            .client
            .post(MESSAGES_API_URL)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
            .header("x-api-key", &self.api_key)
            .json(&request)
            .send()
            .await
            .map_err(|e| ChatRsError::AnthropicError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(ChatRsError::AnthropicError(format!(
                "API error {}: {}",
                status, error_text
            )));
        }

        Ok(self.parse_sse_stream(response).await)
    }

    async fn prompt(&self, message: &str) -> Result<String, ChatRsError> {
        let request = AnthropicRequest {
            model: self.model,
            messages: vec![AnthropicMessage {
                role: "user",
                content: message,
            }],
            max_tokens: self.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS),
            temperature: self.temperature,
            system: None,
            stream: None,
        };

        let response = self
            .client
            .post(MESSAGES_API_URL)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
            .header("x-api-key", &self.api_key)
            .json(&request)
            .send()
            .await
            .map_err(|e| ChatRsError::AnthropicError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(ChatRsError::AnthropicError(format!(
                "API error {}: {}",
                status, error_text
            )));
        }

        let anthropic_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| ChatRsError::AnthropicError(format!("Failed to parse response: {}", e)))?;

        let text = anthropic_response
            .content
            .first()
            .filter(|block| block.block_type == "text")
            .map(|block| block.text.clone())
            .ok_or_else(|| ChatRsError::NoResponse)?;

        if let Some(usage) = anthropic_response.usage {
            let usage: ChatRsUsage = usage.into();
            println!("Prompt usage: {:?}", usage);
        }

        Ok(text)
    }
}

/// Anthropic API request message
#[derive(Debug, Serialize)]
struct AnthropicMessage<'a> {
    role: &'a str,
    content: &'a str,
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
}

/// Anthropic API response content block
#[derive(Debug, Deserialize)]
struct AnthropicContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: String,
}

/// Anthropic API response usage
#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: Option<u32>,
    output_tokens: Option<u32>,
}

impl From<AnthropicUsage> for ChatRsUsage {
    fn from(usage: AnthropicUsage) -> Self {
        ChatRsUsage {
            input_tokens: usage.input_tokens,
            output_tokens: usage.output_tokens,
            cost: None,
        }
    }
}

/// Anthropic API response
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContentBlock>,
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
#[serde(tag = "type")]
enum AnthropicStreamEvent {
    #[serde(rename = "message_start")]
    MessageStart { message: AnthropicStreamResponse },
    #[serde(rename = "content_block_start")]
    ContentBlockStart {
        // index: usize,
        content_block: AnthropicContentBlock,
    },
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta {
        // index: usize,
        delta: AnthropicDelta,
    },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop {
        // index: usize,
    },
    #[serde(rename = "message_delta")]
    MessageDelta {
        // delta: AnthropicMessageDelta,
        usage: Option<AnthropicUsage>,
    },
    #[serde(rename = "message_stop")]
    MessageStop,
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "error")]
    Error { error: AnthropicError },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum AnthropicDelta {
    #[serde(rename = "text_delta")]
    TextDelta { text: String },
    #[serde(rename = "input_json_delta")]
    InputJsonDelta {
        // partial_json: String
    },
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
