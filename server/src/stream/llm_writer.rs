use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use fred::prelude::{KeysInterface, StreamsInterface};
use rocket::futures::StreamExt;
use serde::Serialize;

use crate::{
    db::models::ChatRsToolCall,
    provider::{LlmApiStream, LlmError, LlmUsage},
};

/// Interval at which chunks are flushed to Redis.
const FLUSH_INTERVAL: Duration = Duration::from_millis(500);
/// Max accumulated size of the text before it is automatically flushed to Redis.
const MAX_CHUNK_SIZE: usize = 1000;
/// Expiration in seconds set on the Redis stream (normally, the Redis stream will be deleted before this)
const STREAM_EXPIRE: i64 = 30;

/// Utility for processing an incoming LLM response stream and intermittently
/// writing chunks to a Redis stream.
#[derive(Debug)]
pub struct LlmStreamWriter {
    redis: fred::prelude::Pool,
    /// The current chunk of data being processed.
    current_chunk: ChunkState,
    /// Accumulated text response from the assistant.
    complete_text: Option<String>,
    /// Accumulated tool calls from the assistant.
    tool_calls: Option<Vec<ChatRsToolCall>>,
    /// Accumulated errors during the stream from the LLM provider.
    errors: Option<Vec<LlmError>>,
    /// Accumulated usage information from the LLM provider.
    usage: Option<LlmUsage>,
}

/// Internal state
#[derive(Debug, Default)]
struct ChunkState {
    text: Option<String>,
    tool_calls: Option<Vec<ChatRsToolCall>>,
    error: Option<String>,
}

/// Chunk of the LLM response stored in the Redis stream.
#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
enum RedisStreamChunk {
    Start,
    Text(String),
    ToolCall(String),
    Error(String),
    End,
}
impl From<RedisStreamChunk> for HashMap<String, String> {
    /// Converts a `RedisStreamChunk` into a hash map, suitable for the Redis client.
    fn from(chunk: RedisStreamChunk) -> Self {
        let value = serde_json::to_value(chunk).unwrap_or_default();
        serde_json::from_value(value).unwrap_or_default()
    }
}

impl LlmStreamWriter {
    pub fn new(redis: &fred::prelude::Pool) -> Self {
        LlmStreamWriter {
            redis: redis.clone(),
            current_chunk: ChunkState::default(),
            complete_text: None,
            tool_calls: None,
            errors: None,
            usage: None,
        }
    }

    /// Process the incoming stream from the LLM provider, intermittently
    /// flushing chunks to a Redis stream, and return the final accumulated response.
    pub async fn process_stream(
        mut self,
        stream_key: &str,
        mut stream: LlmApiStream,
    ) -> (
        Option<String>,
        Option<Vec<ChatRsToolCall>>,
        Option<LlmUsage>,
        Option<Vec<LlmError>>,
    ) {
        if let Err(e) = self.notify_start_of_redis_stream(&stream_key).await {
            self.errors.get_or_insert_default().push(LlmError::Redis(e));
        };

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
                    self.process_error(err);
                }
            }

            if self.should_flush(&last_flush_time) {
                self.flush_and_reset(&stream_key).await;
                last_flush_time = Instant::now();
            }
        }

        if let Err(e) = self.notify_end_of_redis_stream(&stream_key).await {
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

    fn process_error(&mut self, err: LlmError) {
        self.current_chunk.error = Some(err.to_string());
        self.errors.get_or_insert_default().push(err);
    }

    fn should_flush(&self, last_flush_time: &Instant) -> bool {
        if self.current_chunk.tool_calls.is_some() || self.current_chunk.error.is_some() {
            return true;
        }
        if let Some(ref text) = self.current_chunk.text {
            return text.len() > MAX_CHUNK_SIZE || last_flush_time.elapsed() > FLUSH_INTERVAL;
        }
        return false;
    }

    async fn flush_and_reset(&mut self, stream_key: &str) {
        let chunk_state = std::mem::take(&mut self.current_chunk);

        let mut chunks: Vec<RedisStreamChunk> = Vec::with_capacity(2);
        if let Some(text) = chunk_state.text {
            chunks.push(RedisStreamChunk::Text(text));
        }
        if let Some(tool_calls) = chunk_state.tool_calls {
            chunks.extend(tool_calls.into_iter().map(|tc| {
                RedisStreamChunk::ToolCall(serde_json::to_string(&tc).unwrap_or_default())
            }));
        }
        if let Some(error) = chunk_state.error {
            chunks.push(RedisStreamChunk::Error(error));
        }

        let entries = chunks.into_iter().map(|chunk| chunk.into()).collect();
        let _ = self.add_to_redis_stream(stream_key, entries).await;
    }

    async fn notify_start_of_redis_stream(
        &mut self,
        stream_key: &str,
    ) -> Result<(), fred::prelude::Error> {
        let entries = vec![RedisStreamChunk::Start.into()];
        self.add_to_redis_stream(stream_key, entries).await
    }

    async fn notify_end_of_redis_stream(
        &mut self,
        stream_key: &str,
    ) -> Result<(), fred::prelude::Error> {
        let entries = vec![RedisStreamChunk::End.into()];
        self.add_to_redis_stream(stream_key, entries).await
    }

    async fn add_to_redis_stream(
        &mut self,
        stream_key: &str,
        entries: Vec<HashMap<String, String>>,
    ) -> Result<(), fred::prelude::Error> {
        let pipeline = self.redis.next().pipeline();
        for entry in entries {
            let _: () = pipeline.xadd(stream_key, false, None, "*", entry).await?;
        }
        let _: () = pipeline.expire(stream_key, STREAM_EXPIRE, None).await?;
        pipeline.all().await
    }
}
