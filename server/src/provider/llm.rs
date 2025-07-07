use llm::{
    builder::{LLMBackend, LLMBuilder},
    chat::{ChatMessage, ImageMime},
    error::LLMError,
    LLMProvider,
};
use rocket::{async_trait, futures::TryStreamExt};

use crate::{
    db::models::{ChatRsFileContentType, ChatRsMessageRole},
    provider::{ChatRsError, ChatRsProvider, ChatRsProviderMessage, ChatRsStream},
};

/// LLM API chat provider via the `llm` crate
pub struct LlmApiProvider<'a> {
    backend: LLMBackend,
    api_key: String,
    base_url: Option<&'a str>,

    model: &'a str,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

impl<'a> LlmApiProvider<'a> {
    pub fn new(
        backend: LLMBackend,
        api_key: String,
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

    fn get_llm(&self, stream: bool) -> Result<Box<(dyn LLMProvider + 'static)>, LLMError> {
        let mut llm_builder = LLMBuilder::new()
            .backend(self.backend.to_owned())
            .api_key(&self.api_key)
            .model(self.model)
            .stream(stream);
        if let Some(max_tokens) = self.max_tokens {
            llm_builder = llm_builder.max_tokens(max_tokens);
        }
        if let Some(temperature) = self.temperature {
            llm_builder = llm_builder.temperature(temperature);
        }
        if let Some(base_url) = self.base_url {
            llm_builder = llm_builder.base_url(base_url);
        }

        llm_builder.build()
    }
}

#[async_trait]
impl<'a> ChatRsProvider for LlmApiProvider<'a> {
    async fn chat_stream(
        &self,
        message_history: Vec<ChatRsProviderMessage>,
    ) -> Result<ChatRsStream, ChatRsError> {
        let llm = self.get_llm(true)?;
        let messages: Vec<ChatMessage> = message_history
            .into_iter()
            .filter_map(|message| match message {
                ChatRsProviderMessage::Message(text) => match text.role {
                    ChatRsMessageRole::User => {
                        Some(ChatMessage::user().content(text.content).build())
                    }
                    ChatRsMessageRole::Assistant => {
                        Some(ChatMessage::assistant().content(text.content).build())
                    }
                    ChatRsMessageRole::System => None,
                },
                ChatRsProviderMessage::Attachment(file) => match file.content_type {
                    ChatRsFileContentType::Pdf => {
                        let bytes = std::fs::read(file.path).ok()?;
                        Some(ChatMessage::user().pdf(bytes).build())
                    }
                    _ => {
                        let bytes = std::fs::read(file.path).ok()?;
                        Some(
                            ChatMessage::user()
                                .image(file.content_type.try_into().ok()?, bytes)
                                .build(),
                        )
                    }
                },
            })
            .collect();

        let stream = llm.chat_stream(&messages).await?.map_err(|e| e.into());

        Ok(Box::pin(stream))
    }

    async fn prompt(&self, request: &str) -> Result<String, ChatRsError> {
        let llm = self.get_llm(false)?;
        let messages = vec![ChatMessage::user().content(request).build()];

        llm.chat(&messages)
            .await?
            .text()
            .ok_or(ChatRsError::ChatError("No text response".to_owned()))
    }
}

impl TryFrom<ChatRsFileContentType> for ImageMime {
    type Error = &'static str;

    fn try_from(value: ChatRsFileContentType) -> Result<Self, Self::Error> {
        match value {
            ChatRsFileContentType::Jpg => Ok(ImageMime::JPEG),
            ChatRsFileContentType::Png => Ok(ImageMime::PNG),
            ChatRsFileContentType::Gif => Ok(ImageMime::GIF),
            ChatRsFileContentType::Webp => Ok(ImageMime::WEBP),
            ChatRsFileContentType::Pdf => Err("Not an image"),
        }
    }
}
