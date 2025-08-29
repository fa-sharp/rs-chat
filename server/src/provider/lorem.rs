//! Lorem ipsum LLM provider (for testing)

use std::pin::Pin;
use std::time::Duration;

use rocket::futures::Stream;
use rocket_okapi::JsonSchema;
use tokio::time::{interval, Interval};

use crate::{
    db::models::ChatRsMessage,
    provider::{
        LlmApiProvider, LlmError, LlmProviderOptions, LlmStream, LlmStreamChunk,
        LlmStreamChunkResult, LlmStreamError, LlmTool,
    },
    provider_models::LlmModel,
};

/// A test/dummy provider that streams 'lorem ipsum...' and emits test errors during the stream
#[derive(Debug, Clone)]
pub struct LoremProvider {
    pub config: LoremConfig,
}

#[derive(Debug, Clone, JsonSchema)]
pub struct LoremConfig {
    pub interval: u32,
}

impl LoremProvider {
    pub fn new() -> Self {
        LoremProvider {
            config: LoremConfig { interval: 400 },
        }
    }
}

struct LoremStream {
    words: Vec<&'static str>,
    index: usize,
    interval: Interval,
}
impl Stream for LoremStream {
    type Item = LlmStreamChunkResult;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        if self.index >= self.words.len() {
            return std::task::Poll::Ready(None);
        }

        match Pin::new(&mut self.interval).poll_tick(cx) {
            std::task::Poll::Ready(_) => {
                let word = self.words[self.index];
                self.index += 1;
                if self.index == 0 || self.index % 10 != 0 {
                    std::task::Poll::Ready(Some(Ok(LlmStreamChunk::Text(word.to_owned()))))
                } else {
                    std::task::Poll::Ready(Some(Err(LlmStreamError::ProviderError(
                        "Test error".into(),
                    ))))
                }
            }
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

#[rocket::async_trait]
impl LlmApiProvider for LoremProvider {
    async fn chat_stream(
        &self,
        _messages: Vec<ChatRsMessage>,
        _tools: Option<Vec<LlmTool>>,
        _options: &LlmProviderOptions,
    ) -> Result<LlmStream, LlmError> {
        let lorem_words = vec![
            "Lorem ipsum ",
            "dolor sit ",
            "amet, consectetur ",
            "adipiscing elit, ",
            "sed do",
            " eiusmod tempor",
            " incididunt ut",
            " labore et",
            " dolore magna ",
            "aliqua. Ut ",
            "enim ad ",
            "minim veniam,",
            " quis nostrud",
            " exercitation ullamco",
            " laboris nisi ",
            "ut aliquip ",
            "ex ea ",
            "commodo consequat. ",
            "Duis aute ",
            "irure dolor ",
            "in reprehenderit ",
            "in voluptate ",
            "velit esse ",
            "cillum dolore ",
            "eu fugiat ",
            "nulla pariatur.",
        ];

        let stream: LlmStream = Box::pin(LoremStream {
            words: lorem_words,
            index: 0,
            interval: interval(Duration::from_millis(self.config.interval.into())),
        });

        tokio::time::sleep(Duration::from_millis(1000)).await;

        Ok(stream)
    }

    async fn prompt(
        &self,
        _request: &str,
        _options: &LlmProviderOptions,
    ) -> Result<String, LlmError> {
        Ok("Lorem ipsum".to_string())
    }

    async fn list_models(&self) -> Result<Vec<LlmModel>, LlmError> {
        Ok(vec![])
    }
}
