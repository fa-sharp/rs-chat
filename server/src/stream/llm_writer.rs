use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use fred::prelude::{KeysInterface, StreamsInterface};
use rocket::futures::StreamExt;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    db::models::ChatRsToolCall,
    provider::{LlmApiStream, LlmError, LlmUsage},
    stream::get_chat_stream_key,
};

/// Interval at which chunks are flushed to the Redis stream.
const FLUSH_INTERVAL: Duration = Duration::from_millis(500);
/// Max accumulated size of the text chunk before it is automatically flushed to Redis.
const MAX_CHUNK_SIZE: usize = 200;
/// Expiration in seconds set on the Redis stream (normally, the Redis stream will be deleted before this)
const STREAM_EXPIRE: i64 = 30;
/// Timeout waiting for data from the LLM stream.
const LLM_TIMEOUT: Duration = Duration::from_secs(20);
/// Interval for sending ping messages to the Redis stream.
const PING_INTERVAL: Duration = Duration::from_secs(2);

/// Utility for processing an incoming LLM response stream and writing to a Redis stream.
#[derive(Debug)]
pub struct LlmStreamWriter {
    redis: fred::prelude::Pool,
    /// The key of the Redis stream.
    key: String,
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
    Ping,
    Text(String),
    ToolCall(String),
    Error(String),
    Cancel,
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
    pub fn new(redis: &fred::prelude::Pool, user_id: &Uuid, session_id: &Uuid) -> Self {
        LlmStreamWriter {
            redis: redis.clone(),
            key: get_chat_stream_key(user_id, session_id),
            current_chunk: ChunkState::default(),
            complete_text: None,
            tool_calls: None,
            errors: None,
            usage: None,
        }
    }

    /// Check if the Redis stream already exists.
    pub async fn exists(&self) -> Result<bool, fred::prelude::Error> {
        let first_entry: Option<()> = self.redis.xread(Some(1), None, &self.key, "0-0").await?;
        Ok(first_entry.is_some())
    }

    /// Create the Redis stream and write a `start` entry.
    pub async fn start(&self) -> Result<(), fred::prelude::Error> {
        let entry: HashMap<String, String> = RedisStreamChunk::Start.into();
        let pipeline = self.redis.next().pipeline();
        let _: () = pipeline.xadd(&self.key, false, None, "*", entry).await?;
        let _: () = pipeline.expire(&self.key, STREAM_EXPIRE, None).await?;
        pipeline.all().await
    }

    /// Cancel the current stream by adding a `cancel` event to the stream and then deleting it from Redis
    /// (not using a pipeline since we need to ensure the `cancel` event is processed before deleting the stream).
    pub async fn cancel(&self) -> Result<(), fred::prelude::Error> {
        let entry: HashMap<String, String> = RedisStreamChunk::Cancel.into();
        let _: () = self.redis.xadd(&self.key, true, None, "*", entry).await?;
        self.redis.del(&self.key).await
    }

    /// Add an `end` event to notify clients that the stream has ended, and then
    /// delete the stream from Redis.
    pub async fn end(&self) -> Result<(), fred::prelude::Error> {
        let entry: HashMap<String, String> = RedisStreamChunk::End.into();
        let pipeline = self.redis.next().pipeline();
        let _: () = pipeline.xadd(&self.key, true, None, "*", entry).await?;
        let _: () = pipeline.del(&self.key).await?;
        pipeline.all().await
    }

    /// Process the incoming stream from the LLM provider, intermittently flushing
    /// chunks to a Redis stream, and return the final accumulated response.
    pub async fn process(
        &mut self,
        mut stream: LlmApiStream,
    ) -> (
        Option<String>,
        Option<Vec<ChatRsToolCall>>,
        Option<LlmUsage>,
        Option<Vec<String>>,
        bool,
    ) {
        let ping_handle = self.start_ping_task();

        let mut last_flush_time = Instant::now();
        let mut cancelled = false;
        loop {
            match tokio::time::timeout(LLM_TIMEOUT, stream.next()).await {
                Ok(Some(Ok(chunk))) => {
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
                Ok(Some(Err(err))) => self.process_error(err),
                Ok(None) => break,
                Err(_) => {
                    self.process_error(LlmError::StreamTimeout);
                    break;
                }
            }

            if self.should_flush(&last_flush_time) {
                if let Err(err) = self.flush_chunk().await {
                    if matches!(err, LlmError::StreamNotFound) {
                        self.errors.get_or_insert_default().push(err);
                        cancelled = true;
                        break;
                    }
                    self.process_error(err);
                }
                last_flush_time = Instant::now();
            }
        }
        ping_handle.abort();

        let complete_text = self.complete_text.take();
        let tool_calls = self.tool_calls.take();
        let usage = self.usage.take();
        let errors = self.errors.take().map(|e| {
            e.into_iter()
                .map(|e| e.to_string())
                .collect::<Vec<String>>()
        });
        (complete_text, tool_calls, usage, errors, cancelled)
    }

    fn process_text(&mut self, text: &str) {
        self.current_chunk
            .text
            .get_or_insert_with(|| String::with_capacity(MAX_CHUNK_SIZE))
            .push_str(text);
        self.complete_text
            .get_or_insert_with(|| String::with_capacity(1024))
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
        let text = self.current_chunk.text.as_ref();
        text.is_some_and(|t| t.len() > MAX_CHUNK_SIZE) || last_flush_time.elapsed() > FLUSH_INTERVAL
    }

    async fn flush_chunk(&mut self) -> Result<(), LlmError> {
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
        if chunks.is_empty() {
            return Ok(());
        }

        let entries = chunks.into_iter().map(|chunk| chunk.into()).collect();
        self.add_to_redis_stream(entries).await
    }

    /// Adds a new entry to the Redis stream. Returns a `LlmError::StreamNotFound` error if the stream has been deleted or cancelled.
    async fn add_to_redis_stream(
        &self,
        entries: Vec<HashMap<String, String>>,
    ) -> Result<(), LlmError> {
        let pipeline = self.redis.next().pipeline();
        for entry in entries {
            let _: () = pipeline
                .xadd(&self.key, true, ("MAXLEN", "~", 500), "*", entry)
                .await?;
        }
        let _: () = pipeline.expire(&self.key, STREAM_EXPIRE, None).await?;
        let res: Vec<fred::prelude::Value> = pipeline.all().await?;

        // Check for `nil` responses indicating the stream has been deleted/cancelled
        if res.iter().any(|r| matches!(r, fred::prelude::Value::Null)) {
            Err(LlmError::StreamNotFound)
        } else {
            Ok(())
        }
    }

    /// Start task that pings the Redis stream every `PING_INTERVAL` seconds
    fn start_ping_task(&self) -> tokio::task::JoinHandle<()> {
        let redis = self.redis.clone();
        let key = self.key.to_owned();
        let ping_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(PING_INTERVAL);
            loop {
                interval.tick().await;
                let entry: HashMap<String, String> = RedisStreamChunk::Ping.into();
                let res: Result<(), fred::error::Error> =
                    redis.xadd(&key, true, None, "*", entry).await;
                if res.is_err() {
                    break;
                }
            }
        });
        ping_handle
    }
}
