//! OpenAI (and OpenAI compatible) LLM provider

use futures::StreamExt;

use crate::llm::{
    error::LlmRequestError,
    interface::{LlmPromptResponse, LlmProvider, LlmStreamingResponse},
    providers::utils::{self, llm_api_request},
    types::{LlmChatRequest, LlmPrompt, LlmUsage},
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
    config: OpenAIProviderConfig,
}

impl OpenAIProvider {
    pub fn new(http_client: &reqwest::Client, config: OpenAIProviderConfig) -> Self {
        Self {
            client: http_client.clone(),
            config,
        }
    }

    pub fn openai(http_client: &reqwest::Client, api_key: impl Into<String>) -> Self {
        Self::new(http_client, OpenAIProviderConfig::openai(api_key))
    }

    pub fn openrouter(http_client: &reqwest::Client, api_key: impl Into<String>) -> Self {
        Self::new(http_client, OpenAIProviderConfig::openrouter(api_key))
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
    fn prompt<'r>(&'r self, prompt: LlmPrompt<'r>) -> LlmPromptResponse<'r> {
        let policy = OpenAIRequestPolicy::new(self.config.flavor);
        let request = OpenAIRequest {
            model: &prompt.options.model,
            messages: vec![OpenAIMessage {
                role: "user",
                content: Some(vec![OpenAIContent::Text { text: prompt.text }]),
                ..Default::default()
            }],
            max_tokens: policy.max_tokens(prompt.options.max_tokens),
            max_completion_tokens: policy.max_completion_tokens(prompt.options.max_tokens),
            store: policy.store(),
            ..Default::default()
        };

        Box::pin(async move {
            let provider_name = self.config.flavor.name();
            let response = llm_api_request(
                &self.client,
                provider_name,
                &format!("{}/chat/completions", self.config.base_url),
                &self.config.api_key,
                &request,
            )
            .await?;
            let mut response: OpenAIResponse = response.json().await.map_err(|err| {
                LlmRequestError::Provider(format!("Failed to parse response: {err}"))
            })?;

            let text = response
                .choices
                .get_mut(0)
                .and_then(|choice| choice.message.as_mut())
                .and_then(|message| message.content.take())
                .ok_or(LlmRequestError::NoContent)?;
            if let Some(usage) = response.usage {
                let usage: LlmUsage = usage.into();
                tracing::info!("Prompt usage: {usage:?}");
            }

            Ok(text)
        })
    }

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
            let response = llm_api_request(
                &self.client,
                provider_name,
                &format!("{}/chat/completions", self.config.base_url),
                &self.config.api_key,
                &request,
            )
            .await?;

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
