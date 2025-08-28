//! OpenAI (and OpenAI compatible) LLM provider

use rocket::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    db::models::{ChatRsMessage, ChatRsMessageRole, ChatRsToolCall},
    provider::{
        LlmApiProvider, LlmApiProviderSharedOptions, LlmApiStream, LlmError, LlmPendingToolCall,
        LlmStreamChunk, LlmTool, LlmUsage,
    },
    provider_models::{LlmModel, ModelsDevService, ModelsDevServiceProvider},
};

const OPENAI_API_BASE_URL: &str = "https://api.openai.com/v1";
const OPENROUTER_API_BASE_URL: &str = "https://openrouter.ai/api/v1";

/// OpenAI chat provider
#[derive(Debug, Clone)]
pub struct OpenAIProvider {
    client: reqwest::Client,
    redis: fred::clients::Client,
    api_key: String,
    base_url: String,
}

impl OpenAIProvider {
    pub fn new(
        http_client: &reqwest::Client,
        redis: &fred::clients::Client,
        api_key: &str,
        base_url: Option<&str>,
    ) -> Self {
        Self {
            client: http_client.clone(),
            redis: redis.clone(),
            api_key: api_key.to_owned(),
            base_url: base_url.unwrap_or(OPENAI_API_BASE_URL).to_owned(),
        }
    }

    fn build_messages<'a>(&self, messages: &'a [ChatRsMessage]) -> Vec<OpenAIMessage<'a>> {
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

    fn build_tools<'a>(&self, tools: &'a [LlmTool]) -> Vec<OpenAITool<'a>> {
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

    async fn parse_sse_stream(
        &self,
        mut response: reqwest::Response,
        tools: Option<Vec<LlmTool>>,
    ) -> LlmApiStream {
        let stream = async_stream::stream! {
            let mut buffer = String::new();
            let mut tool_calls: Vec<OpenAIStreamToolCall> = Vec::new();

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
                                                if let Some(text) = delta.content {
                                                    yield Ok(LlmStreamChunk::Text(text));
                                                }

                                                if let Some(tool_calls_delta) = delta.tool_calls {
                                                    for tool_call_delta in tool_calls_delta {
                                                        yield Ok(LlmStreamChunk::PendingToolCall(LlmPendingToolCall {
                                                            index: tool_call_delta.index,
                                                            tool_name: tool_call_delta.function.name.clone(),
                                                        }));
                                                        if let Some(tc) = tool_calls.iter_mut().find(|tc| tc.index == tool_call_delta.index) {
                                                            if let Some(function_arguments) = tool_call_delta.function.arguments {
                                                                *tc.function.arguments.get_or_insert_default() += &function_arguments;
                                                            }
                                                        } else {
                                                            tool_calls.push(tool_call_delta);
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        // Yield usage information if available
                                        if let Some(usage) = response.usage {
                                            yield Ok(LlmStreamChunk::Usage(usage.into()));
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
                        yield Err(LlmError::OpenAIError(format!("Stream error: {}", e)));
                        break;
                    }
                }
            }

            if let Some(rs_chat_tools) = tools {
                if !tool_calls.is_empty() {
                    yield Ok(LlmStreamChunk::ToolCalls(tool_calls.into_iter().filter_map(|tc| tc.convert(&rs_chat_tools)).collect()));
                }
            }

            rocket::debug!("SSE stream ended");
        };

        Box::pin(stream)
    }
}

#[async_trait]
impl LlmApiProvider for OpenAIProvider {
    async fn chat_stream(
        &self,
        messages: Vec<ChatRsMessage>,
        tools: Option<Vec<LlmTool>>,
        options: &LlmApiProviderSharedOptions,
    ) -> Result<LlmApiStream, LlmError> {
        let openai_messages = self.build_messages(&messages);
        let openai_tools = tools.as_ref().map(|t| self.build_tools(t));

        let request = OpenAIRequest {
            model: &options.model,
            messages: openai_messages,
            max_tokens: options.max_tokens,
            temperature: options.temperature,
            stream: Some(true),
            stream_options: Some(OpenAIStreamOptions {
                include_usage: true,
            }),
            tools: openai_tools,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("authorization", format!("Bearer {}", self.api_key))
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| LlmError::OpenAIError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(LlmError::OpenAIError(format!(
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
        let request = OpenAIRequest {
            model: &options.model,
            messages: vec![OpenAIMessage {
                role: "user",
                content: Some(message),
                ..Default::default()
            }],
            max_tokens: options.max_tokens,
            temperature: options.temperature,
            ..Default::default()
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("authorization", format!("Bearer {}", self.api_key))
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| LlmError::OpenAIError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(LlmError::OpenAIError(format!(
                "API error {}: {}",
                status, error_text
            )));
        }

        let mut openai_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| LlmError::OpenAIError(format!("Failed to parse response: {}", e)))?;

        let text = openai_response
            .choices
            .get_mut(0)
            .and_then(|choice| choice.message.as_mut())
            .and_then(|message| message.content.take())
            .ok_or(LlmError::NoResponse)?;

        if let Some(usage) = openai_response.usage {
            let usage: LlmUsage = usage.into();
            println!("Prompt usage: {:?}", usage);
        }

        Ok(text)
    }

    async fn list_models(&self) -> Result<Vec<LlmModel>, LlmError> {
        let models_service = ModelsDevService::new(&self.redis, &self.client);
        let models = models_service
            .list_models({
                match self.base_url.as_str() {
                    OPENROUTER_API_BASE_URL => ModelsDevServiceProvider::OpenRouter,
                    _ => ModelsDevServiceProvider::OpenAI,
                }
            })
            .await?;

        Ok(models)
    }
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
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OpenAITool<'a>>>,
}

/// OpenAI API request message
#[derive(Debug, Default, Serialize)]
struct OpenAIMessage<'a> {
    role: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAIToolCall<'a>>>,
}

/// OpenAI tool definition
#[derive(Debug, Serialize)]
struct OpenAITool<'a> {
    #[serde(rename = "type")]
    tool_type: &'a str,
    function: OpenAIToolFunction<'a>,
}

/// OpenAI tool function definition
#[derive(Debug, Serialize)]
struct OpenAIToolFunction<'a> {
    name: &'a str,
    description: &'a str,
    parameters: &'a serde_json::Value,
    strict: bool,
}

/// OpenAI tool call in messages
#[derive(Debug, Serialize)]
struct OpenAIToolCall<'a> {
    id: &'a str,
    #[serde(rename = "type")]
    tool_type: &'a str,
    function: OpenAIToolCallFunction<'a>,
}

/// OpenAI tool call function in messages
#[derive(Debug, Serialize)]
struct OpenAIToolCallFunction<'a> {
    name: &'a str,
    arguments: String,
}

/// OpenAI API request stream options
#[derive(Debug, Serialize)]
struct OpenAIStreamOptions {
    include_usage: bool,
}

/// OpenAI API response
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

/// OpenAI API streaming response
#[derive(Debug, Deserialize)]
struct OpenAIStreamResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
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
    tool_calls: Option<Vec<OpenAIStreamToolCall>>,
}

/// OpenAI streaming tool call
#[derive(Debug, Deserialize)]
struct OpenAIStreamToolCall {
    id: Option<String>,
    index: usize,
    function: OpenAIStreamToolCallFunction,
}

impl OpenAIStreamToolCall {
    /// Convert OpenAI tool call format to ChatRsToolCall, add tool ID
    fn convert(self, rs_chat_tools: &[LlmTool]) -> Option<ChatRsToolCall> {
        let id = self.id?;
        let tool_name = self.function.name;
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
    name: String,
    arguments: Option<String>,
}

/// OpenAI API response usage
#[derive(Debug, Deserialize)]
struct OpenAIUsage {
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
