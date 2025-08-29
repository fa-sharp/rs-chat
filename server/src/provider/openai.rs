//! OpenAI (and OpenAI compatible) LLM provider

mod request;
mod response;

use rocket::{async_stream, async_trait, futures::StreamExt};

use crate::{
    db::models::ChatRsMessage,
    provider::{
        utils::get_sse_events, LlmApiProvider, LlmError, LlmProviderOptions, LlmStream,
        LlmStreamChunk, LlmTool, LlmUsage,
    },
    provider_models::{LlmModel, ModelsDevService, ModelsDevServiceProvider},
};

use {
    request::{
        build_openai_messages, build_openai_tools, OpenAIMessage, OpenAIRequest,
        OpenAIStreamOptions,
    },
    response::{parse_openai_event, OpenAIResponse, OpenAIStreamToolCall},
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
}

#[async_trait]
impl LlmApiProvider for OpenAIProvider {
    async fn chat_stream(
        &self,
        messages: Vec<ChatRsMessage>,
        tools: Option<Vec<LlmTool>>,
        options: &LlmProviderOptions,
    ) -> Result<LlmStream, LlmError> {
        let openai_messages = build_openai_messages(&messages);
        let openai_tools = tools.as_ref().map(|t| build_openai_tools(t));

        let request = OpenAIRequest {
            model: &options.model,
            messages: openai_messages,
            max_tokens: (options.max_tokens.is_some() && self.base_url != OPENAI_API_BASE_URL)
                .then(|| options.max_tokens.expect("already checked for Some value")),
            // OpenAI official API has deprecated `max_tokens` for `max_completion_tokens`
            max_completion_tokens: (options.max_tokens.is_some()
                && self.base_url == OPENAI_API_BASE_URL)
                .then(|| options.max_tokens.expect("already checked for Some value")),
            temperature: options.temperature,
            store: (self.base_url == OPENAI_API_BASE_URL).then_some(false),
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
            .map_err(|e| LlmError::ProviderError(format!("OpenAI request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(LlmError::ProviderError(format!(
                "OpenAI API error {}: {}",
                status, error_text
            )));
        }

        let stream = async_stream::stream! {
            let mut sse_event_stream = get_sse_events(response);
            let mut tool_calls: Vec<OpenAIStreamToolCall> = Vec::new();
            while let Some(event) = sse_event_stream.next().await {
                match event {
                    Ok(event) => {
                        for chunk in parse_openai_event(event, &mut tool_calls) {
                            yield chunk;
                        }
                    }
                    Err(e) => yield Err(e),
                }
            }
            if !tool_calls.is_empty() {
                if let Some(llm_tools) = tools {
                    let converted = tool_calls
                        .into_iter()
                        .filter_map(|tc| tc.convert(&llm_tools))
                        .collect();
                    yield Ok(LlmStreamChunk::ToolCalls(converted));
                }
            }
        };

        Ok(stream.boxed())
    }

    async fn prompt(
        &self,
        message: &str,
        options: &LlmProviderOptions,
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
            .map_err(|e| LlmError::ProviderError(format!("OpenAI request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(LlmError::ProviderError(format!(
                "OpenAI API error {}: {}",
                status, error_text
            )));
        }

        let mut openai_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| LlmError::ProviderError(format!("Failed to parse response: {}", e)))?;

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
