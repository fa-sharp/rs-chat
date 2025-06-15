use llm::{
    builder::{LLMBackend, LLMBuilder},
    chat::ChatMessage,
};
use rocket::{async_trait, futures::TryStreamExt};

use crate::{
    db::models::{ChatRsMessage, ChatRsMessageRole},
    provider::{ChatRsError, ChatRsProvider, ChatRsStream},
};

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

    async fn chat_stream(
        &self,
        input: Option<&str>,
        context: Option<Vec<ChatRsMessage>>,
    ) -> Result<ChatRsStream, ChatRsError> {
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
        let llm = llm_builder.build()?;

        let mut messages: Vec<ChatMessage> = match context {
            None => vec![],
            Some(message_history) => message_history
                .into_iter()
                .filter_map(|message| match message.role {
                    ChatRsMessageRole::User => {
                        Some(ChatMessage::user().content(message.content).build())
                    }
                    ChatRsMessageRole::Assistant => {
                        Some(ChatMessage::assistant().content(message.content).build())
                    }
                    ChatRsMessageRole::System => None,
                })
                .collect(),
        };
        if let Some(user_message) = input {
            messages.push(ChatMessage::user().content(user_message).build());
        }

        let stream = llm.chat_stream(&messages).await?.map_err(|e| e.into());

        Ok(Box::pin(stream))
    }
}
