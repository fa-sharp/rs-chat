//! Anthropic LLM provider

use futures::StreamExt;

use crate::llm::{
    error::LlmRequestError,
    interface::*,
    providers::utils,
    types::{LlmChatRequest, LlmPrompt},
};

mod request;
mod response;

use {request::*, response::*};

const MESSAGES_API_URL: &str = "https://api.anthropic.com/v1/messages";
const API_VERSION: &str = "2023-06-01";
const REQ_ID_HEADER: &'static str = "request-id";
const DEFAULT_MAX_TOKENS: u32 = 4096;

/// Anthropic chat provider
#[derive(Debug, Clone)]
pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
}

impl AnthropicProvider {
    pub fn new(http_client: &reqwest::Client, api_key: impl Into<String>) -> Self {
        Self {
            client: http_client.clone(),
            api_key: api_key.into(),
        }
    }
}

impl LlmProvider for AnthropicProvider {
    fn prompt<'r>(&'r self, prompt: LlmPrompt<'r>) -> LlmPromptResponse<'r> {
        let request = AnthropicRequest {
            model: &prompt.options.model,
            messages: vec![AnthropicMessage {
                role: "user",
                content: vec![AnthropicContentBlock::Text { text: prompt.text }],
            }],
            max_tokens: prompt.options.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS),
            temperature: prompt.options.temperature,
            system: None,
            stream: None,
            tools: None,
        };

        Box::pin(async move {
            let raw_response = utils::llm_api_request(
                self.client
                    .post(MESSAGES_API_URL)
                    .header("anthropic-version", API_VERSION)
                    .header("x-api-key", &self.api_key)
                    .json(&request),
                "Anthropic",
            )
            .await?;
            let request_id = utils::extract_header(&raw_response, REQ_ID_HEADER);
            let mut response: AnthropicResponse = raw_response.json().await?;

            let text = response
                .content
                .get_mut(0)
                .and_then(|block| match block {
                    AnthropicResponseContentBlock::Text { text } => Some(std::mem::take(text)),
                    _ => None,
                })
                .ok_or_else(|| LlmRequestError::NoContent)?;

            Ok(LlmResponse {
                text,
                usage: response.usage.map(Into::into).unwrap_or_default(),
                meta: LlmResponseMeta::new(request_id),
            })
        })
    }

    fn stream_chat<'r>(&'r self, req: LlmChatRequest<'r>) -> LlmStreamingResponse<'r> {
        let (anthropic_messages, system_prompt) = build_anthropic_messages(&req.messages);
        // let anthropic_tools = tools.as_ref().map(|t| build_anthropic_tools(t));
        let request = AnthropicRequest {
            model: &req.options.model,
            messages: anthropic_messages,
            max_tokens: req.options.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS),
            temperature: req.options.temperature,
            system: system_prompt,
            stream: Some(true),
            // tools: anthropic_tools,
            ..Default::default()
        };

        Box::pin(async move {
            let response = utils::llm_api_request(
                self.client
                    .post(MESSAGES_API_URL)
                    .header("anthropic-version", API_VERSION)
                    .header("x-api-key", &self.api_key)
                    .json(&request),
                "Anthropic",
            )
            .await?;
            let request_id = utils::extract_header(&response, REQ_ID_HEADER);

            let stream = async_stream::stream! {
                let mut sse_event_stream = utils::get_sse_events(response);
                // let mut tool_calls = Vec::new();
                while let Some(event_result) = sse_event_stream.next().await {
                    match event_result {
                        Ok(event) => {
                            if let Some(chunk) = parse_anthropic_event(event) {
                                yield chunk;
                            }
                        },
                        Err(e) => yield Err(e),
                    }
                }
            };

            Ok((stream.boxed(), LlmResponseMeta::new(request_id)))
        })
    }
}
