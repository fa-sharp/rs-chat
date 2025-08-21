use std::time::{Duration, Instant};

use fred::prelude::StreamsInterface;
use rocket::futures::StreamExt;

use crate::{
    db::models::ChatRsToolCall,
    provider::{LlmApiStream, LlmError, LlmUsage},
};

const MAX_CHUNK_SIZE: usize = 1000;
const MAX_FLUSH_TIME: Duration = Duration::from_millis(500);

/// Utility struct for processing an incoming LLM stream and intermittently
/// flushing the data to a Redis stream.
#[derive(Debug, Default)]
pub struct LlmStreamProcessor {
    redis: fred::prelude::Client,
    /// The current chunk of data being processed.
    current_chunk: RedisStreamChunkData,
    /// Accumulated text response from the assistant.
    complete_text: Option<String>,
    /// Accumulated tool calls from the assistant.
    tool_calls: Option<Vec<ChatRsToolCall>>,
    /// Accumulated errors during the stream from the assistant.
    errors: Option<Vec<LlmError>>,
    /// Accumulated usage information from the assistant.
    usage: Option<LlmUsage>,
}

#[derive(Debug)]
enum RedisStreamChunk {
    Data(RedisStreamChunkData),
    End,
}

#[derive(Debug, Default)]
struct RedisStreamChunkData {
    text: Option<String>,
    tool_calls: Option<Vec<ChatRsToolCall>>,
    error: Option<String>,
}

impl LlmStreamProcessor {
    pub fn new(redis: &fred::prelude::Client) -> Self {
        LlmStreamProcessor {
            redis: redis.clone(),
            ..Default::default()
        }
    }

    /// Process the incoming stream from the LLM provider, intermittently
    /// flush to Redis stream, and return the accumulated results.
    pub async fn process_llm_stream(
        mut self,
        stream_key: &str,
        mut stream: LlmApiStream,
    ) -> (
        Option<String>,
        Option<Vec<ChatRsToolCall>>,
        Option<LlmUsage>,
        Option<Vec<LlmError>>,
    ) {
        let mut last_flush_time = Instant::now();

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(chunk) => {
                    if let Some(ref text) = chunk.text {
                        self.process_text(text);
                    }
                    if let Some(tool_calls) = chunk.tool_calls {
                        self.process_tool_calls(tool_calls);
                    }
                    if let Some(usage_chunk) = chunk.usage {
                        self.process_usage(usage_chunk);
                    }
                }
                Err(err) => {
                    self.current_chunk.error = Some(err.to_string());
                    self.errors.get_or_insert_default().push(err);
                }
            }

            if self.should_flush(&last_flush_time) {
                self.flush_and_reset_chunk(&stream_key).await;
                last_flush_time = Instant::now();
            }
        }

        if let Err(e) = self.mark_end_of_redis_stream(&stream_key).await {
            self.errors.get_or_insert_default().push(LlmError::Redis(e));
        };

        (self.complete_text, self.tool_calls, self.usage, self.errors)
    }

    fn process_text(&mut self, text: &str) {
        self.current_chunk
            .text
            .get_or_insert_with(|| String::with_capacity(MAX_CHUNK_SIZE + 200))
            .push_str(text);
        self.complete_text
            .get_or_insert_with(|| String::with_capacity(2000))
            .push_str(text);
    }

    fn process_tool_calls(&mut self, tool_calls: Vec<ChatRsToolCall>) {
        self.current_chunk
            .tool_calls
            .get_or_insert_default()
            .extend(tool_calls.clone());
        self.tool_calls.get_or_insert_default().extend(tool_calls);
    }

    fn process_usage(&mut self, usage_chunk: LlmUsage) {
        let usage = self.usage.get_or_insert_default();
        if let Some(input_tokens) = usage_chunk.input_tokens {
            usage.input_tokens = Some(input_tokens);
        }
        if let Some(output_tokens) = usage_chunk.output_tokens {
            usage.output_tokens = Some(output_tokens);
        }
        if let Some(cost) = usage_chunk.cost {
            usage.cost = Some(cost);
        }
    }

    fn should_flush(&self, last_flush_time: &Instant) -> bool {
        // Flush if there are any tool calls or errors
        if self.current_chunk.tool_calls.is_some() || self.current_chunk.error.is_some() {
            return true;
        }
        // Skip flushing if chunk is completely empty
        if self.current_chunk.text.is_none() {
            return false;
        }
        // Check for time and size triggers
        last_flush_time.elapsed() > MAX_FLUSH_TIME
            || self
                .current_chunk
                .text
                .as_ref()
                .is_some_and(|t| t.len() > MAX_CHUNK_SIZE)
    }

    async fn add_to_redis_stream(
        &mut self,
        stream_key: &str,
        data: Vec<(&str, String)>,
    ) -> Result<(), fred::prelude::Error> {
        self.redis.xadd(stream_key, false, None, "*", data).await
    }

    async fn flush_and_reset_chunk(&mut self, stream_key: &str) {
        let chunk = std::mem::take(&mut self.current_chunk);
        if let Ok(data) = RedisStreamChunk::Data(chunk).try_into() {
            let _ = self.add_to_redis_stream(stream_key, data).await;
        }
    }

    async fn mark_end_of_redis_stream(
        &mut self,
        stream_key: &str,
    ) -> Result<(), fred::prelude::Error> {
        let data = RedisStreamChunk::End.try_into().expect("Should convert");
        self.add_to_redis_stream(stream_key, data).await
    }
}

impl TryFrom<RedisStreamChunk> for Vec<(&str, String)> {
    type Error = serde_json::Error;

    /// Converts a `RedisStreamChunk` into a vector of key-value pairs, suitable for the Redis client.
    fn try_from(chunk: RedisStreamChunk) -> Result<Self, Self::Error> {
        match chunk {
            RedisStreamChunk::Data(data) => {
                let mut vec = Vec::with_capacity(3);
                vec.push(("type", "data".into()));
                if let Some(text) = data.text {
                    vec.push(("text", text));
                }
                if let Some(tool_calls) = data.tool_calls {
                    vec.push(("tool_calls", serde_json::to_string(&tool_calls)?));
                }
                if let Some(error) = data.error {
                    vec.push(("error", error));
                }
                Ok(vec)
            }
            RedisStreamChunk::End => Ok(vec![("type", "end".into())]),
        }
    }
}
