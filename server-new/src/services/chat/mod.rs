use futures::Stream;

use crate::services::{
    chat::error::ChatError,
    llm::{
        interface::{LlmProvider, LlmStreamChunkResult},
        providers::OpenAIProvider,
        types::{LlmChatOptions, LlmChatRequest, LlmMessage, LlmUserMessage},
    },
};

mod error;

pub struct ChatService<'a> {
    http_client: &'a reqwest::Client,
    redis: &'a fred::clients::Client,
}

impl<'a> ChatService<'a> {
    pub fn new(http_client: &'a reqwest::Client, redis: &'a fred::clients::Client) -> Self {
        Self { http_client, redis }
    }

    pub async fn test_chat(&self) -> Result<impl Stream<Item = LlmStreamChunkResult>, ChatError> {
        let provider: Box<dyn LlmProvider> =
            Box::new(OpenAIProvider::new(self.http_client, self.redis, "", None));
        let messages = vec![LlmMessage::User(LlmUserMessage {
            text: "Hello!".into(),
            ..Default::default()
        })];
        let request = LlmChatRequest {
            messages: messages,
            options: LlmChatOptions {
                model: "gpt-5-mini".into(),
                ..Default::default()
            },
        };

        let response = provider.stream_chat(&request).await?;

        Ok(response)
    }
}
