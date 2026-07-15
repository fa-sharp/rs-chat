//! OpenAI (and OpenAI compatible) LLM provider

use futures::StreamExt;

use crate::{
    db::models::OpenAISubtype,
    llm::{
        error::LlmRequestError,
        interface::*,
        providers::utils,
        types::{LlmChatRequest, LlmPrompt},
    },
};

mod request;
mod response;

use {request::*, response::*};

const OPENAI_API_BASE_URL: &str = "https://api.openai.com/v1";
const OPENROUTER_API_BASE_URL: &str = "https://openrouter.ai/api/v1";

impl OpenAISubtype {
    fn name(&self) -> &'static str {
        match self {
            Self::OpenAI => "OpenAI",
            Self::OpenRouter => "OpenRouter",
        }
    }
    fn default_base_url(&self) -> &'static str {
        match self {
            Self::OpenAI => OPENAI_API_BASE_URL,
            Self::OpenRouter => OPENROUTER_API_BASE_URL,
        }
    }
    fn req_id_header(&self) -> &'static str {
        match self {
            OpenAISubtype::OpenAI => "X-Request-Id",
            OpenAISubtype::OpenRouter => "X-Generation-Id",
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
    subtype: OpenAISubtype,
    api_key: String,
    base_url: String,
}

impl OpenAIProviderConfig {
    pub fn new(
        subtype: OpenAISubtype,
        api_key: impl Into<String>,
        base_url: Option<impl Into<String>>,
    ) -> Self {
        Self {
            subtype,
            api_key: api_key.into(),
            base_url: base_url
                .map(Into::into)
                .unwrap_or_else(|| subtype.default_base_url().to_owned())
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
}

#[derive(Debug, Clone, Copy)]
struct OpenAIRequestPolicy {
    subtype: OpenAISubtype,
}

impl OpenAIRequestPolicy {
    fn new(subtype: OpenAISubtype) -> Self {
        Self { subtype }
    }

    fn max_tokens(self, max_tokens: Option<u32>) -> Option<u32> {
        (!self.subtype.use_max_completion_tokens())
            .then_some(max_tokens)
            .flatten()
    }

    fn max_completion_tokens(self, max_tokens: Option<u32>) -> Option<u32> {
        self.subtype
            .use_max_completion_tokens()
            .then_some(max_tokens)
            .flatten()
    }

    fn store(self) -> Option<bool> {
        self.subtype.include_store_false().then_some(false)
    }

    fn stream_options(self) -> Option<OpenAIStreamOptions> {
        self.subtype
            .include_usage_stream_options()
            .then_some(OpenAIStreamOptions {
                include_usage: true,
            })
    }
}

impl LlmProvider for OpenAIProvider {
    fn prompt<'r>(&'r self, prompt: LlmPrompt<'r>) -> LlmPromptResponse<'r> {
        let policy = OpenAIRequestPolicy::new(self.config.subtype);
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
            let provider_name = self.config.subtype.name();
            let raw_response = utils::llm_api_request(
                self.client
                    .post(format!("{}/chat/completions", self.config.base_url))
                    .bearer_auth(&self.config.api_key)
                    .json(&request),
                provider_name,
                Some(self.config.subtype.req_id_header()),
            )
            .await?;
            let req_id = utils::extract_header(&raw_response, self.config.subtype.req_id_header());
            let mut response: OpenAIResponse = raw_response.json().await?;

            let text = response
                .choices
                .get_mut(0)
                .and_then(|choice| choice.message.as_mut())
                .and_then(|message| message.content.take())
                .ok_or(LlmRequestError::NoContent)?;

            Ok(LlmResponse {
                text,
                usage: response.usage.map(Into::into).unwrap_or_default(),
                meta: LlmResponseMeta::new(req_id),
            })
        })
    }

    fn stream_chat<'r>(&'r self, req: LlmChatRequest<'r>) -> LlmStreamingResponse<'r> {
        let policy = OpenAIRequestPolicy::new(self.config.subtype);
        let openai_messages = build_openai_messages(req.messages);
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
        let provider_name = self.config.subtype.name();

        Box::pin(async move {
            let response = utils::llm_api_request(
                self.client
                    .post(format!("{}/chat/completions", self.config.base_url))
                    .bearer_auth(&self.config.api_key)
                    .json(&request),
                provider_name,
                Some(self.config.subtype.req_id_header()),
            )
            .await?;
            let req_id = utils::extract_header(&response, self.config.subtype.req_id_header());

            let stream = async_stream::stream! {
                let mut sse_event_stream = utils::get_sse_events(response);
                // let mut tool_calls: Vec<OpenAIStreamToolCall> = Vec::new();
                while let Some(event) = sse_event_stream.next().await {
                    match event {
                        Ok(event) => {
                            for chunk in parse_openai_event(event) {
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

            Ok((stream.boxed(), LlmResponseMeta::new(req_id)))
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
