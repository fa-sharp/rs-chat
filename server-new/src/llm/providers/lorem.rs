//! Lorem ipsum LLM provider (for testing)

use std::{pin::Pin, time::Duration};

use futures::Stream;
use tokio::time::{Interval, interval};

use crate::llm::{
    error::LlmStreamChunkError,
    interface::*,
    types::{LlmChatRequest, LlmPrompt, LlmUsage},
};

/// A test/dummy provider that streams 'lorem ipsum...' and emits test errors during the stream
#[derive(Debug, Clone)]
pub struct LoremProvider {
    pub interval: u32,
}

impl LoremProvider {
    pub fn new() -> Self {
        LoremProvider { interval: 400 }
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

        match self.interval.poll_tick(cx) {
            std::task::Poll::Ready(_) => {
                let word = self.words[self.index];
                self.index += 1;
                if self.index == 0 || self.index % 10 != 0 {
                    std::task::Poll::Ready(Some(Ok(LlmStreamChunk::Text(word.to_owned()))))
                } else {
                    std::task::Poll::Ready(Some(Err(LlmStreamChunkError::Provider(
                        "Test error".into(),
                    ))))
                }
            }
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

impl LlmProvider for LoremProvider {
    fn prompt<'r>(&'r self, prompt: LlmPrompt<'r>) -> LlmPromptResponse<'r> {
        let response = LlmResponse {
            text: "Lorem ipsum".into(),
            usage: LlmUsage {
                input_tokens: Some((prompt.text.len() / 4) as u32),
                output_tokens: Some(4),
                ..Default::default()
            },
            ..Default::default()
        };

        Box::pin(async { Ok(response) })
    }

    fn stream_chat<'r>(&'r self, _request: LlmChatRequest<'r>) -> LlmStreamingResponse<'r> {
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

        Box::pin(async move {
            let stream: LlmStream = Box::pin(LoremStream {
                words: lorem_words,
                index: 0,
                interval: interval(Duration::from_millis(self.interval.into())),
            });
            tokio::time::sleep(Duration::from_millis(1000)).await; // Simulate initial request latency

            Ok((stream, LlmResponseMeta::default()))
        })
    }
}
