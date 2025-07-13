use llm::async_trait;
use openrouter_rs::{
    api::{
        chat::{ChatCompletionRequest, Message},
        completion::CompletionRequest,
    },
    types::Role,
    OpenRouterClient,
};
use rocket::futures::StreamExt;

use crate::{
    db::models::{ChatRsMessage, ChatRsMessageRole},
    provider::{ChatRsError, ChatRsProvider, ChatRsStream, ChatRsStreamChunk, ChatRsUsage},
};

/// OpenRouter chat provider via the `openrouter-rs` crate
pub struct OpenRouterProvider<'a> {
    api_key: String,

    model: &'a str,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

impl<'a> OpenRouterProvider<'a> {
    pub fn new(
        api_key: String,
        model: &'a str,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    ) -> Self {
        Self {
            api_key,
            model,
            max_tokens,
            temperature,
        }
    }
}

#[async_trait]
impl ChatRsProvider for OpenRouterProvider<'_> {
    async fn chat_stream(&self, messages: Vec<ChatRsMessage>) -> Result<ChatRsStream, ChatRsError> {
        let client = OpenRouterClient::builder().api_key(&self.api_key).build()?;
        let messages: Vec<Message> = messages
            .into_iter()
            .map(|msg| Message::new(msg.role.into(), &msg.content))
            .collect();

        let request = ChatCompletionRequest::builder()
            .model(self.model)
            .messages(messages)
            .temperature(self.temperature.unwrap_or(0.7).into())
            .max_tokens(self.max_tokens.unwrap_or(1000))
            .build()?;

        let stream = client
            .stream_chat_completion(&request)
            .await?
            .map(|chunk| match chunk {
                Ok(res) => {
                    let text = res
                        .choices
                        .first()
                        .and_then(|choice| choice.content())
                        .unwrap_or_default()
                        .to_owned();
                    let usage = res.usage.map(|usage| ChatRsUsage {
                        input_tokens: Some(usage.prompt_tokens),
                        output_tokens: Some(usage.completion_tokens),
                    });
                    Ok(ChatRsStreamChunk { text, usage })
                }
                Err(err) => Err(err.into()),
            });

        Ok(Box::pin(stream))
    }

    async fn prompt(&self, input: &str) -> Result<String, ChatRsError> {
        let client = OpenRouterClient::builder().api_key(&self.api_key).build()?;

        let request = CompletionRequest::builder()
            .model(self.model)
            .prompt(input)
            .temperature(self.temperature.unwrap_or(0.7).into())
            .max_tokens(self.max_tokens.unwrap_or(1000))
            .build()?;

        let response = client.send_completion_request(&request).await?;
        let content = response
            .choices
            .first()
            .and_then(|choice| choice.content())
            .ok_or(ChatRsError::ChatError("No text response".to_owned()))?;

        Ok(content.to_string())
    }
}

impl From<ChatRsMessageRole> for Role {
    fn from(role: ChatRsMessageRole) -> Self {
        match role {
            ChatRsMessageRole::User => Role::User,
            ChatRsMessageRole::Assistant => Role::Assistant,
            ChatRsMessageRole::System => Role::System,
        }
    }
}
