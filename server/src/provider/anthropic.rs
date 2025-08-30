//! Anthropic LLM provider

mod request;
mod response;

use rocket::{async_stream, async_trait, futures::StreamExt};

use crate::{
    db::models::ChatRsMessage,
    provider::{
        models::{LlmModel, ModelsDevService, ModelsDevServiceProvider},
        utils::get_sse_events,
        LlmApiProvider, LlmError, LlmProviderOptions, LlmStream, LlmTool, LlmUsage,
        DEFAULT_MAX_TOKENS,
    },
};

use {
    request::{
        build_anthropic_messages, build_anthropic_tools, AnthropicContentBlock, AnthropicMessage,
        AnthropicRequest,
    },
    response::{parse_anthropic_event, AnthropicResponse, AnthropicResponseContentBlock},
};

const MESSAGES_API_URL: &str = "https://api.anthropic.com/v1/messages";
const API_VERSION: &str = "2023-06-01";

/// Anthropic chat provider
#[derive(Debug, Clone)]
pub struct AnthropicProvider {
    client: reqwest::Client,
    redis: fred::clients::Client,
    api_key: String,
}

impl AnthropicProvider {
    pub fn new(
        http_client: &reqwest::Client,
        redis: &fred::clients::Client,
        api_key: &str,
    ) -> Self {
        Self {
            client: http_client.clone(),
            redis: redis.clone(),
            api_key: api_key.to_string(),
        }
    }
}

#[async_trait]
impl LlmApiProvider for AnthropicProvider {
    async fn chat_stream(
        &self,
        messages: Vec<ChatRsMessage>,
        tools: Option<Vec<LlmTool>>,
        options: &LlmProviderOptions,
    ) -> Result<LlmStream, LlmError> {
        let (anthropic_messages, system_prompt) = build_anthropic_messages(&messages);
        let anthropic_tools = tools.as_ref().map(|t| build_anthropic_tools(t));
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
            .header("x-api-key", &self.api_key)
            .json(&request)
            .send()
            .await
            .map_err(|e| LlmError::ProviderError(format!("Anthropic request failed: {}", e)))?;
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(LlmError::ProviderError(format!(
                "Anthropic API error {}: {}",
                status, error_text
            )));
        }

        let stream = async_stream::stream! {
            let mut sse_event_stream = get_sse_events(response);
            let mut tool_calls = Vec::new();
            while let Some(event_result) = sse_event_stream.next().await {
                match event_result {
                    Ok(event) => {
                        if let Some(chunk) = parse_anthropic_event(event, tools.as_ref(), &mut tool_calls) {
                            yield chunk;
                        }
                    },
                    Err(e) => yield Err(e),
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
            .header("x-api-key", &self.api_key)
            .json(&request)
            .send()
            .await
            .map_err(|e| LlmError::ProviderError(format!("Anthropic request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(LlmError::ProviderError(format!(
                "Anthropic API error {}: {}",
                status, error_text
            )));
        }

        let mut anthropic_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| LlmError::ProviderError(format!("Failed to parse response: {}", e)))?;
        let text = anthropic_response
            .content
            .get_mut(0)
            .and_then(|block| match block {
                AnthropicResponseContentBlock::Text { text } => Some(std::mem::take(text)),
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
        let models_service = ModelsDevService::new(&self.redis, &self.client);
        let models = models_service
            .list_models(ModelsDevServiceProvider::Anthropic)
            .await?;

        Ok(models)
    }
}
