use llm::{
    builder::{LLMBackend, LLMBuilder},
    chat::ChatMessage,
};
use rocket::{
    async_trait,
    futures::{stream, TryStreamExt},
};

use crate::provider::{ChatRsProvider, ChatRsStream};

/// LLM API chat provider via the `llm` crate
pub struct LlmApiProvider<'a> {
    backend: LLMBackend,
    api_key: &'a str,
    base_url: Option<&'a str>,

    model: &'a str,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

impl<'a> LlmApiProvider<'a> {
    pub fn new(
        backend: LLMBackend,
        api_key: &'a str,
        model: &'a str,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    ) -> Self {
        Self {
            backend,
            api_key,
            model,
            max_tokens,
            temperature,
            base_url: None,
        }
    }
}

#[async_trait]
impl<'a> ChatRsProvider for LlmApiProvider<'a> {
    fn name(&self) -> &'static str {
        "llm_api"
    }

    fn display_name(&self) -> &'static str {
        "LLM API"
    }

    async fn chat_stream(&self, input: &str, _context: Option<String>) -> ChatRsStream {
        let mut llm_builder = LLMBuilder::new()
            .backend(self.backend.to_owned())
            .api_key(self.api_key)
            .model(self.model)
            .stream(true);

        if let Some(max_tokens) = self.max_tokens {
            llm_builder = llm_builder.max_tokens(max_tokens);
        }
        if let Some(temperature) = self.temperature {
            llm_builder = llm_builder.temperature(temperature);
        }
        if let Some(base_url) = self.base_url {
            llm_builder = llm_builder.base_url(base_url);
        }

        let llm = match llm_builder.build() {
            Ok(llm) => llm,
            Err(e) => return Box::pin(stream::once(async { Err(e.into()) })),
        };
        let messages = vec![ChatMessage::user().content(input).build()];
        let stream = match llm.chat_stream(&messages).await {
            Ok(stream) => stream.map_err(|e| e.into()),
            Err(e) => return Box::pin(stream::once(async { Err(e.into()) })),
        };

        Box::pin(stream)
    }
}
