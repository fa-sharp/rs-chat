//! Ollama LLM provider

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

const CHAT_API_URL: &str = "/api/chat";
const COMPLETION_API_URL: &str = "/api/generate";

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

impl LlmProvider for OllamaProvider {
    fn prompt<'r>(&'r self, prompt: LlmPrompt<'r>) -> LlmPromptResponse<'r> {
        let ollama_options = OllamaOptions {
            temperature: prompt.options.temperature,
            num_predict: prompt.options.max_tokens,
            ..Default::default()
        };
        let request = OllamaCompletionRequest {
            model: &prompt.options.model,
            prompt: prompt.text,
            stream: Some(false),
            options: Some(ollama_options),
        };

        Box::pin(async move {
            let res: OllamaCompletionResponse = utils::llm_api_request(
                self.client
                    .post(format!("{}{}", self.base_url, COMPLETION_API_URL))
                    .json(&request),
                "Ollama",
                None,
            )
            .await?
            .json()
            .await?;

            if res.response.is_empty() {
                return Err(LlmRequestError::NoContent);
            }

            Ok(LlmResponse {
                usage: res.usage().unwrap_or_default(),
                text: res.response,
                ..Default::default()
            })
        })
    }

    fn stream_chat<'r>(&'r self, req: LlmChatRequest<'r>) -> LlmStreamingResponse<'r> {
        let ollama_messages = build_ollama_messages(req.messages);
        // let ollama_tools = tools.as_ref().map(|t| build_ollama_tools(t));
        let ollama_options = OllamaOptions {
            temperature: req.options.temperature,
            num_predict: req.options.max_tokens,
            ..Default::default()
        };
        let request = OllamaChatRequest {
            model: &req.options.model,
            messages: ollama_messages,
            // tools: ollama_tools,
            stream: Some(true),
            options: Some(ollama_options),
        };

        Box::pin(async move {
            let response = utils::llm_api_request(
                self.client
                    .post(format!("{}{}", self.base_url, CHAT_API_URL))
                    .json(&request),
                "Ollama",
                None,
            )
            .await?;
            let stream = async_stream::stream! {
                let mut json_stream = utils::get_json_events(response);
                // let mut tool_calls: Vec<OllamaToolCallResponse> = Vec::new();
                while let Some(event) = json_stream.next().await {
                    match event {
                        Ok(event) => {
                            for chunk in parse_ollama_event(event) {
                                yield Ok(chunk);
                            }
                        }
                        Err(e) => yield Err(e),
                    }
                }
                // if !tool_calls.is_empty() {
                //     if let Some(llm_tools) = tools {
                //         let converted = tool_calls
                //             .into_iter()
                //             .filter_map(|tc| tc.function.convert(&llm_tools))
                //             .collect();
                //         yield Ok(LlmStreamChunk::ToolCalls(converted));
                //     }
                // }
            };

            Ok((stream.boxed(), LlmResponseMeta::default()))
        })
    }
}
