//! OpenAI (and OpenAI compatible) LLM provider

use futures::StreamExt;

use crate::services::llm::{
    error::LlmRequestError,
    interface::{LlmProvider, LlmStreamingResponse},
    providers::utils,
    types::LlmChatRequest,
};

mod request;
mod response;

use {request::*, response::*};

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

impl LlmProvider for OpenAIProvider {
    fn stream_chat<'r>(&'r self, req: &'r LlmChatRequest) -> LlmStreamingResponse<'r> {
        let openai_messages = build_openai_messages(&req.messages);
        // let openai_tools = tools.as_ref().map(|t| build_openai_tools(t));
        //
        let request = OpenAIRequest {
            model: &req.options.model,
            messages: openai_messages,
            // OpenAI official API deprecated `max_tokens` for `max_completion_tokens`
            max_tokens: match req.options.max_tokens {
                Some(max_tokens) if self.base_url != OPENAI_API_BASE_URL => Some(max_tokens),
                _ => None,
            },
            max_completion_tokens: match req.options.max_tokens {
                Some(max_tokens) if self.base_url == OPENAI_API_BASE_URL => Some(max_tokens),
                _ => None,
            },
            temperature: req.options.temperature,
            store: (self.base_url == OPENAI_API_BASE_URL).then_some(false),
            stream: Some(true),
            stream_options: Some(OpenAIStreamOptions {
                include_usage: true,
            }),
            // tools: openai_tools,
            // modalities: options.modalities.as_ref(),
            ..Default::default()
        };

        Box::pin(async move {
            let response = self
                .client
                .post(format!("{}/chat/completions", self.base_url))
                .header("authorization", format!("Bearer {}", self.api_key))
                .header("content-type", "application/json")
                .json(&request)
                .send()
                .await
                .map_err(|e| LlmRequestError::Provider(format!("OpenAI request failed: {}", e)))?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                return Err(LlmRequestError::Provider(format!(
                    "OpenAI API error {}: {}",
                    status, error_text
                )));
            }

            let stream = async_stream::stream! {
                let mut sse_event_stream = utils::get_sse_events(response);
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
                // if !tool_calls.is_empty() {
                //     if let Some(llm_tools) = tools {
                //         let converted = tool_calls
                //             .into_iter()
                //             .filter_map(|tc| tc.convert(&llm_tools))
                //             .collect();
                //         yield Ok(LlmStreamChunk::ToolCalls(converted));
                //     }
                // }
            };

            Ok(stream.boxed())
        })
    }

    // async fn prompt(
    //     &self,
    //     message: &str,
    //     options: &LlmProviderOptions,
    // ) -> Result<String, LlmError> {
    //     let request = OpenAIRequest {
    //         model: &options.model,
    //         messages: vec![OpenAIMessage {
    //             role: "user",
    //             content: Some(vec![OpenAIContent::Text { text: message }]),
    //             ..Default::default()
    //         }],
    //         max_tokens: options.max_tokens,
    //         temperature: options.temperature,
    //         store: (self.base_url == OPENAI_API_BASE_URL).then_some(false),
    //         ..Default::default()
    //     };

    //     let response = self
    //         .client
    //         .post(format!("{}/chat/completions", self.base_url))
    //         .header("authorization", format!("Bearer {}", self.api_key))
    //         .header("content-type", "application/json")
    //         .json(&request)
    //         .send()
    //         .await
    //         .map_err(|e| LlmError::ProviderError(format!("OpenAI request failed: {}", e)))?;

    //     if !response.status().is_success() {
    //         let status = response.status();
    //         let error_text = response.text().await.unwrap_or_default();
    //         return Err(LlmError::ProviderError(format!(
    //             "OpenAI API error {}: {}",
    //             status, error_text
    //         )));
    //     }

    //     let mut openai_response: OpenAIResponse = response
    //         .json()
    //         .await
    //         .map_err(|e| LlmError::ProviderError(format!("Failed to parse response: {}", e)))?;

    //     let text = openai_response
    //         .choices
    //         .get_mut(0)
    //         .and_then(|choice| choice.message.as_mut())
    //         .and_then(|message| message.content.take())
    //         .ok_or(LlmError::NoResponse)?;

    //     if let Some(usage) = openai_response.usage {
    //         let usage: LlmUsage = usage.into();
    //         println!("Prompt usage: {:?}", usage);
    //     }

    //     Ok(text)
    // }

    // async fn list_models(&self) -> Result<Vec<LlmModel>, LlmError> {
    //     let models = models::ModelsDevService::new(&self.redis, &self.client)
    //         .list_models({
    //             match self.base_url.as_str() {
    //                 OPENROUTER_API_BASE_URL => models::ModelsDevServiceProvider::OpenRouter,
    //                 _ => models::ModelsDevServiceProvider::OpenAI,
    //             }
    //         })
    //         .await?;

    //     Ok(models)
    // }
}
