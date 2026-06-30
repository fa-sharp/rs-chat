//! OpenAI (and OpenAI compatible) LLM provider

use futures::StreamExt;

use crate::llm::{
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

/// OpenAI-compatible provider behavior variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAIProviderFlavor {
    OpenAI,
    OpenRouter,
}
impl OpenAIProviderFlavor {
    fn name(self) -> &'static str {
        match self {
            Self::OpenAI => "OpenAI",
            Self::OpenRouter => "OpenRouter",
        }
    }
    fn default_base_url(self) -> &'static str {
        match self {
            Self::OpenAI => OPENAI_API_BASE_URL,
            Self::OpenRouter => OPENROUTER_API_BASE_URL,
        }
    }
    fn use_max_completion_tokens(self) -> bool {
        self == Self::OpenAI
    }
    fn include_store_false(self) -> bool {
        self == Self::OpenAI
    }
    fn include_usage_stream_options(self) -> bool {
        true
    }
}

/// Configuration for OpenAI-compatible providers.
#[derive(Debug, Clone)]
pub struct OpenAIProviderConfig {
    flavor: OpenAIProviderFlavor,
    api_key: String,
    base_url: String,
}

impl OpenAIProviderConfig {
    pub fn openai(api_key: impl Into<String>) -> Self {
        Self::new(OpenAIProviderFlavor::OpenAI, api_key, None::<String>)
    }

    pub fn openrouter(api_key: impl Into<String>) -> Self {
        Self::new(OpenAIProviderFlavor::OpenRouter, api_key, None::<String>)
    }

    pub fn new(
        flavor: OpenAIProviderFlavor,
        api_key: impl Into<String>,
        base_url: Option<impl Into<String>>,
    ) -> Self {
        Self {
            flavor,
            api_key: api_key.into(),
            base_url: base_url
                .map(Into::into)
                .unwrap_or_else(|| flavor.default_base_url().to_owned())
                .trim_end_matches('/')
                .to_owned(),
        }
    }
}

/// OpenAI-compatible chat provider.
#[derive(Debug, Clone)]
pub struct OpenAIProvider {
    client: reqwest::Client,
    _redis: fred::clients::Client,
    config: OpenAIProviderConfig,
}

impl OpenAIProvider {
    pub fn new(
        http_client: &reqwest::Client,
        redis: &fred::clients::Client,
        config: OpenAIProviderConfig,
    ) -> Self {
        Self {
            client: http_client.clone(),
            _redis: redis.clone(),
            config,
        }
    }

    pub fn openai(
        http_client: &reqwest::Client,
        redis: &fred::clients::Client,
        api_key: impl Into<String>,
    ) -> Self {
        Self::new(http_client, redis, OpenAIProviderConfig::openai(api_key))
    }

    pub fn openrouter(
        http_client: &reqwest::Client,
        redis: &fred::clients::Client,
        api_key: impl Into<String>,
    ) -> Self {
        Self::new(
            http_client,
            redis,
            OpenAIProviderConfig::openrouter(api_key),
        )
    }
}

#[derive(Debug, Clone, Copy)]
struct OpenAIRequestPolicy {
    flavor: OpenAIProviderFlavor,
}

impl OpenAIRequestPolicy {
    fn new(flavor: OpenAIProviderFlavor) -> Self {
        Self { flavor }
    }

    fn max_tokens(self, max_tokens: Option<u32>) -> Option<u32> {
        (!self.flavor.use_max_completion_tokens())
            .then_some(max_tokens)
            .flatten()
    }

    fn max_completion_tokens(self, max_tokens: Option<u32>) -> Option<u32> {
        self.flavor
            .use_max_completion_tokens()
            .then_some(max_tokens)
            .flatten()
    }

    fn store(self) -> Option<bool> {
        self.flavor.include_store_false().then_some(false)
    }

    fn stream_options(self) -> Option<OpenAIStreamOptions> {
        self.flavor
            .include_usage_stream_options()
            .then_some(OpenAIStreamOptions {
                include_usage: true,
            })
    }
}

impl LlmProvider for OpenAIProvider {
    fn stream_chat<'r>(&'r self, req: LlmChatRequest<'r>) -> LlmStreamingResponse<'r> {
        let policy = OpenAIRequestPolicy::new(self.config.flavor);
        let openai_messages = build_openai_messages(&req.messages);
        // let openai_tools = tools.as_ref().map(|t| build_openai_tools(t));
        //
        let request = OpenAIRequest {
            model: &req.options.model,
            messages: openai_messages,
            max_tokens: policy.max_tokens(req.options.max_tokens),
            max_completion_tokens: policy.max_completion_tokens(req.options.max_tokens),
            temperature: req.options.temperature,
            store: policy.store(),
            stream: Some(true),
            stream_options: policy.stream_options(),
            // tools: openai_tools,
            // modalities: options.modalities.as_ref(),
            ..Default::default()
        };
        let provider_name = self.config.flavor.name();

        Box::pin(async move {
            let response = self
                .client
                .post(format!("{}/chat/completions", self.config.base_url))
                .header("authorization", format!("Bearer {}", self.config.api_key))
                .header("content-type", "application/json")
                .json(&request)
                .send()
                .await
                .map_err(|e| {
                    LlmRequestError::Provider(format!("{provider_name} request failed: {e}"))
                })?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                return Err(LlmRequestError::Provider(format!(
                    "{provider_name} API error {status}: {error_text}",
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
