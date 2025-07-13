use rocket::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    db::models::{ChatRsMessage, ChatRsMessageRole},
    provider::{ChatRsError, ChatRsProvider, ChatRsStream, ChatRsStreamChunk, ChatRsUsage},
};

/// OpenAI chat provider
pub struct OpenAIProvider<'a> {
    client: reqwest::Client,
    api_key: String,
    model: &'a str,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    base_url: &'a str,
}

impl<'a> OpenAIProvider<'a> {
    pub fn new(
        http_client: &reqwest::Client,
        api_key: &str,
        model: &'a str,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
        base_url: Option<&'a str>,
    ) -> Result<Self, ChatRsError> {
        Ok(Self {
            client: http_client.clone(),
            api_key: api_key.to_string(),
            model,
            max_tokens,
            temperature,
            base_url: base_url.unwrap_or("https://api.openai.com"),
        })
    }

    fn build_messages(&self, messages: &'a [ChatRsMessage]) -> Vec<OpenAIMessage<'a>> {
        messages
            .iter()
            .map(|message| {
                let role = match message.role {
                    ChatRsMessageRole::User => "user",
                    ChatRsMessageRole::Assistant => "assistant",
                    ChatRsMessageRole::System => "system",
                };

                OpenAIMessage {
                    role,
                    content: &message.content,
                }
            })
            .collect()
    }

    async fn parse_sse_stream(&self, mut response: reqwest::Response) -> ChatRsStream {
        let stream = async_stream::stream! {
            let mut buffer = String::new();

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

                                match serde_json::from_str::<OpenAIStreamResponse>(data) {
                                    Ok(mut response) => {
                                        if let Some(choice) = response.choices.pop() {
                                            if let Some(delta) = choice.delta {
                                                if let Some(content) = delta.content {
                                                    yield Ok(ChatRsStreamChunk {
                                                        text: content,
                                                        usage: None,
                                                    });
                                                }
                                            }
                                        }

                                        // Yield usage information if available
                                        if let Some(usage) = response.usage {
                                            yield Ok(ChatRsStreamChunk {
                                                text: String::new(),
                                                usage: Some(usage.into()),
                                            });
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
                        yield Err(ChatRsError::OpenAIError(format!("Stream error: {}", e)));
                        break;
                    }
                }
            }

            rocket::debug!("SSE stream ended");
        };

        Box::pin(stream)
    }
}

#[async_trait]
impl<'a> ChatRsProvider for OpenAIProvider<'a> {
    async fn chat_stream(&self, messages: Vec<ChatRsMessage>) -> Result<ChatRsStream, ChatRsError> {
        let openai_messages = self.build_messages(&messages);

        let request = OpenAIRequest {
            model: self.model,
            messages: openai_messages,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            stream: Some(true),
            stream_options: Some(OpenAIStreamOptions {
                include_usage: true,
            }),
        };

        let response = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .header("authorization", format!("Bearer {}", self.api_key))
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| ChatRsError::OpenAIError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(ChatRsError::OpenAIError(format!(
                "API error {}: {}",
                status, error_text
            )));
        }

        Ok(self.parse_sse_stream(response).await)
    }

    async fn prompt(&self, message: &str) -> Result<String, ChatRsError> {
        let request = OpenAIRequest {
            model: self.model,
            messages: vec![OpenAIMessage {
                role: "user",
                content: message,
            }],
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            ..Default::default()
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("authorization", format!("Bearer {}", self.api_key))
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| ChatRsError::OpenAIError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(ChatRsError::OpenAIError(format!(
                "API error {}: {}",
                status, error_text
            )));
        }

        let openai_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| ChatRsError::OpenAIError(format!("Failed to parse response: {}", e)))?;

        let text = openai_response
            .choices
            .first()
            .and_then(|choice| choice.message.as_ref())
            .and_then(|message| message.content.as_ref())
            .ok_or(ChatRsError::NoResponse)?;

        if let Some(usage) = openai_response.usage {
            let usage: ChatRsUsage = usage.into();
            println!("Prompt usage: {:?}", usage);
        }

        Ok(text.clone())
    }
}

/// OpenAI API request message
#[derive(Debug, Serialize)]
struct OpenAIMessage<'a> {
    role: &'a str,
    content: &'a str,
}

/// OpenAI API request body
#[derive(Debug, Default, Serialize)]
struct OpenAIRequest<'a> {
    model: &'a str,
    messages: Vec<OpenAIMessage<'a>>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream_options: Option<OpenAIStreamOptions>,
}

/// OpenAI API request stream options
#[derive(Debug, Serialize)]
struct OpenAIStreamOptions {
    include_usage: bool,
}

/// OpenAI API response choice
#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: Option<OpenAIResponseMessage>,
    delta: Option<OpenAIResponseDelta>,
    // finish_reason: Option<String>,
}

/// OpenAI API response message
#[derive(Debug, Deserialize)]
struct OpenAIResponseMessage {
    // role: String,
    content: Option<String>,
}

/// OpenAI API streaming delta
#[derive(Debug, Deserialize)]
struct OpenAIResponseDelta {
    // role: Option<String>,
    content: Option<String>,
}

/// OpenAI API response usage
#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    // total_tokens: Option<u32>,
}

impl From<OpenAIUsage> for ChatRsUsage {
    fn from(usage: OpenAIUsage) -> Self {
        ChatRsUsage {
            input_tokens: usage.prompt_tokens,
            output_tokens: usage.completion_tokens,
        }
    }
}

/// OpenAI API response
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

/// OpenAI streaming response
#[derive(Debug, Deserialize)]
struct OpenAIStreamResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}
