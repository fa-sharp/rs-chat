//! Ollama LLM provider

mod request;
mod response;

use rocket::{async_stream, async_trait, futures::StreamExt};

use crate::{
    db::models::ChatRsMessage,
    provider::{
        ollama::{
            request::{
                build_ollama_messages, build_ollama_tools, OllamaChatRequest,
                OllamaCompletionRequest, OllamaOptions,
            },
            response::{parse_ollama_event, OllamaCompletionResponse, OllamaToolCall},
        },
        utils::get_sse_events,
        LlmApiProvider, LlmError, LlmProviderOptions, LlmStream, LlmStreamChunk, LlmTool, LlmUsage,
    },
    provider_models::LlmModel,
};

const CHAT_API_URL: &str = "/api/chat";

/// Ollama chat provider
#[derive(Debug, Clone)]
pub struct OllamaProvider {
    client: reqwest::Client,
    base_url: String,
}

impl OllamaProvider {
    pub fn new(http_client: &reqwest::Client, base_url: &str) -> Self {
        Self {
            client: http_client.clone(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }
}

#[async_trait]
impl LlmApiProvider for OllamaProvider {
    async fn chat_stream(
        &self,
        messages: Vec<ChatRsMessage>,
        tools: Option<Vec<LlmTool>>,
        options: &LlmProviderOptions,
    ) -> Result<LlmStream, LlmError> {
        let ollama_messages = build_ollama_messages(&messages);
        let ollama_tools = tools.as_ref().map(|t| build_ollama_tools(t));
        let ollama_options = OllamaOptions {
            temperature: options.temperature,
            num_predict: options.max_tokens,
            ..Default::default()
        };
        let request = OllamaChatRequest {
            model: &options.model,
            messages: ollama_messages,
            tools: ollama_tools,
            stream: Some(true),
            options: Some(ollama_options),
        };

        let response = self
            .client
            .post(format!("{}{}", self.base_url, CHAT_API_URL))
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| LlmError::ProviderError(format!("Ollama request failed: {}", e)))?;
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(LlmError::ProviderError(format!(
                "Ollama API error {}: {}",
                status, error_text
            )));
        }

        let stream = async_stream::stream! {
            let mut sse_event_stream = get_sse_events(response);
            let mut tool_calls: Vec<OllamaToolCall> = Vec::new();
            while let Some(event) = sse_event_stream.next().await {
                match event {
                    Ok(event) => {
                        for chunk in parse_ollama_event(event, &mut tool_calls) {
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
                        .filter_map(|tc| tc.function.convert(&llm_tools))
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
        let ollama_options = OllamaOptions {
            temperature: options.temperature,
            num_predict: options.max_tokens,
            ..Default::default()
        };
        let request = OllamaCompletionRequest {
            model: &options.model,
            prompt: message,
            stream: Some(false),
            options: Some(ollama_options),
        };
        let response = self
            .client
            .post(format!("{}{}", self.base_url, CHAT_API_URL))
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| LlmError::ProviderError(format!("Ollama request failed: {}", e)))?;
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(LlmError::ProviderError(format!(
                "Ollama API error {}: {}",
                status, error_text
            )));
        }

        let ollama_response: OllamaCompletionResponse = response
            .json()
            .await
            .map_err(|e| LlmError::ProviderError(format!("Failed to parse response: {}", e)))?;
        if let Some(usage) = Option::<LlmUsage>::from(&ollama_response) {
            println!("Prompt usage: {:?}", usage);
        }
        if ollama_response.response.is_empty() {
            return Err(LlmError::NoResponse);
        }

        Ok(ollama_response.response)
    }

    async fn list_models(&self) -> Result<Vec<LlmModel>, LlmError> {
        // For now, return an empty list since Ollama doesn't appear in models.dev
        // In a real implementation, you might call Ollama's /api/tags endpoint
        // or maintain a static list of supported models
        Ok(Vec::new())
    }
}
