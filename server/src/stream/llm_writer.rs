use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use fred::prelude::{FredResult, KeysInterface, StreamsInterface};
use rocket::futures::StreamExt;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    db::models::ChatRsToolCall,
    provider::{LlmPendingToolCall, LlmStream, LlmStreamChunk, LlmStreamError, LlmUsage},
    redis::ExclusiveRedisClient,
    stream::get_chat_stream_key,
};

/// Interval at which chunks are flushed to the Redis stream.
const FLUSH_INTERVAL: Duration = Duration::from_millis(300);
/// Max accumulated size of the text chunk before it is automatically flushed to Redis.
const MAX_CHUNK_SIZE: usize = 200;
/// Expiration in seconds set on the Redis stream (normally, the Redis stream will be deleted before this)
const STREAM_EXPIRE: i64 = 30;
/// Timeout waiting for data from the LLM stream.
const LLM_TIMEOUT: Duration = Duration::from_secs(60);
/// Interval for sending ping messages to the Redis stream.
const PING_INTERVAL: Duration = Duration::from_secs(5);

/// Utility for processing an incoming LLM response stream and writing to a Redis stream.
#[derive(Debug)]
pub struct LlmStreamWriter {
    /// Redis client with an exclusive connection.
    redis: ExclusiveRedisClient,
    /// The key of the Redis stream.
    key: String,
    /// The current chunk of data being processed.
    current_chunk: ChunkState,
    /// Accumulated text response from the assistant.
    complete_text: Option<String>,
    /// Accumulated tool calls from the assistant.
    tool_calls: Option<Vec<ChatRsToolCall>>,
    /// Accumulated errors during the stream from the LLM provider.
    errors: Option<Vec<LlmStreamError>>,
    /// Accumulated usage information from the LLM provider.
    usage: Option<LlmUsage>,
}

/// Internal state
#[derive(Debug, Default)]
struct ChunkState {
    text: Option<String>,
    tool_calls: Option<Vec<ChatRsToolCall>>,
    pending_tool_calls: Option<Vec<LlmPendingToolCall>>,
    error: Option<String>,
}

/// Chunk of the LLM response stored in the Redis stream.
#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub(super) enum RedisStreamChunk {
    Start,
    Ping,
    Text(String),
    ToolCall(String),
    PendingToolCall(String),
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
    pub fn new(redis: ExclusiveRedisClient, user_id: &Uuid, session_id: &Uuid) -> Self {
        LlmStreamWriter {
            redis,
            key: get_chat_stream_key(user_id, session_id),
            current_chunk: ChunkState::default(),
            complete_text: None,
            tool_calls: None,
            errors: None,
            usage: None,
        }
    }

    /// Create the Redis stream and write a `start` entry.
    pub async fn start(&self) -> FredResult<()> {
        let entry: HashMap<String, String> = RedisStreamChunk::Start.into();
        let pipeline = self.redis.pipeline();
        let _: () = pipeline.xadd(&self.key, false, None, "*", entry).await?;
        let _: () = pipeline.expire(&self.key, STREAM_EXPIRE, None).await?;
        pipeline.all().await
    }

    /// Add an `end` event to notify clients that the stream has ended, and then
    /// delete the stream from Redis.
    pub async fn end(&self) -> FredResult<()> {
        let entry: HashMap<String, String> = RedisStreamChunk::End.into();
        let pipeline = self.redis.pipeline();
        let _: () = pipeline.xadd(&self.key, true, None, "*", entry).await?;
        let _: () = pipeline.del(&self.key).await?;
        pipeline.all().await
    }

    /// Process the incoming stream from the LLM provider, intermittently flushing
    /// chunks to a Redis stream, and return the final accumulated response.
    pub async fn process(
        &mut self,
        mut stream: LlmStream,
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
                Ok(Some(Ok(chunk))) => match chunk {
                    LlmStreamChunk::Text(text) => self.process_text(&text),
                    LlmStreamChunk::ToolCalls(tool_calls) => self.process_tool_calls(tool_calls),
                    LlmStreamChunk::PendingToolCall(pending_tool_call) => {
                        self.process_pending_tool_call(pending_tool_call)
                    }
                    LlmStreamChunk::Usage(usage) => self.process_usage(usage),
                },
                Ok(Some(Err(err))) => self.process_error(err),
                Ok(None) => {
                    // stream ended
                    self.flush_chunk().await.ok();
                    break;
                }
                Err(_) => {
                    // timed out waiting for provider response
                    self.process_error(LlmStreamError::StreamTimeout);
                    self.flush_chunk().await.ok();
                    break;
                }
            }

            if self.should_flush(&last_flush_time) {
                if let Err(err) = self.flush_chunk().await {
                    if matches!(err, LlmStreamError::StreamCancelled) {
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

    fn process_pending_tool_call(&mut self, tool_call: LlmPendingToolCall) {
        let current_chunk = self
            .current_chunk
            .pending_tool_calls
            .get_or_insert_default();
        if !current_chunk.iter().any(|tc| tc.index == tool_call.index) {
            current_chunk.push(tool_call);
        }
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

    fn process_error(&mut self, err: LlmStreamError) {
        self.current_chunk.error = Some(err.to_string());
        self.errors.get_or_insert_default().push(err);
    }

    fn should_flush(&self, last_flush_time: &Instant) -> bool {
        if self.current_chunk.tool_calls.is_some() || self.current_chunk.error.is_some() {
            return true;
        }
        let text = self.current_chunk.text.as_ref();
        last_flush_time.elapsed() > FLUSH_INTERVAL || text.is_some_and(|t| t.len() > MAX_CHUNK_SIZE)
    }

    /// Flushes the current chunk to the Redis stream. Returns a `LlmStreamError::StreamCancelled` error
    /// if the stream has been deleted or cancelled.
    async fn flush_chunk(&mut self) -> Result<(), LlmStreamError> {
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
        if let Some(pending_tool_calls) = chunk_state.pending_tool_calls {
            chunks.extend(pending_tool_calls.into_iter().map(|tc| {
                RedisStreamChunk::PendingToolCall(serde_json::to_string(&tc).unwrap_or_default())
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

    /// Adds new entries to the Redis stream. Returns a `LlmStreamError::StreamCancelled` error if the
    /// stream has been deleted or cancelled.
    async fn add_to_redis_stream(
        &self,
        entries: Vec<HashMap<String, String>>,
    ) -> Result<(), LlmStreamError> {
        let pipeline = self.redis.pipeline();
        for entry in entries {
            let _: () = pipeline
                .xadd(&self.key, true, ("MAXLEN", "~", 500), "*", entry)
                .await?;
        }
        let res: Vec<fred::prelude::Value> = pipeline.all().await?;

        // Check for `nil` responses indicating the stream has been deleted/cancelled
        if res.iter().any(|r| r.is_null()) {
            Err(LlmStreamError::StreamCancelled)
        } else {
            Ok(())
        }
    }

    /// Start task that pings the Redis stream every `PING_INTERVAL` seconds and extends the expiration time
    fn start_ping_task(&self) -> tokio::task::JoinHandle<()> {
        let redis = self.redis.clone();
        let key = self.key.to_owned();
        let ping_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(PING_INTERVAL);
            loop {
                interval.tick().await;
                let entry: HashMap<String, String> = RedisStreamChunk::Ping.into();
                let pipeline = redis.pipeline();
                let _: FredResult<()> = pipeline.xadd(&key, true, None, "*", entry).await;
                let _: FredResult<()> = pipeline.expire(&key, STREAM_EXPIRE, None).await;
                let res: FredResult<Vec<fred::prelude::Value>> = pipeline.all().await;
                if res.is_err() || res.is_ok_and(|r| r.iter().any(|v| v.is_null())) {
                    break;
                }
            }
        });
        ping_handle
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        provider::{lorem::LoremProvider, LlmApiProvider, LlmProviderOptions},
        redis::{ExclusiveClientManager, ExclusiveClientPool},
        stream::{cancel_current_chat_stream, check_chat_stream_exists},
    };
    use fred::prelude::{Builder, ClientLike, Config};
    use std::time::Duration;

    async fn setup_redis_pool() -> ExclusiveClientPool {
        let config =
            Config::from_url("redis://127.0.0.1:6379").unwrap_or_else(|_| Config::default());
        let pool = Builder::from_config(config)
            .build_pool(1)
            .expect("Failed to build Redis pool");
        pool.init().await.expect("Failed to connect to Redis");

        let manager = ExclusiveClientManager::new(pool.clone());
        let deadpool: ExclusiveClientPool = deadpool::managed::Pool::builder(manager)
            .max_size(3)
            .build()
            .unwrap();

        deadpool
    }

    async fn create_test_writer(
        redis: &ExclusiveClientPool,
        user_id: &Uuid,
        session_id: &Uuid,
    ) -> LlmStreamWriter {
        let client = redis.get().await.expect("Failed to get Redis client");
        LlmStreamWriter::new(ExclusiveRedisClient(client), user_id, session_id)
    }

    #[tokio::test]
    async fn test_stream_writer_basic_functionality() {
        let redis = setup_redis_pool().await;
        let client = redis.get().await.unwrap();
        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let mut writer = create_test_writer(&redis, &user_id, &session_id).await;

        // Create stream
        assert!(writer.start().await.is_ok());
        assert!(check_chat_stream_exists(&client, &user_id, &session_id)
            .await
            .unwrap());

        // Create Lorem provider and get stream
        let lorem = LoremProvider::new();
        let stream = lorem
            .chat_stream(vec![], None, &LlmProviderOptions::default())
            .await
            .expect("Failed to create lorem stream");

        // Process the stream
        let (text, tool_calls, usage, errors, cancelled) = writer.process(stream).await;

        // Verify results
        assert!(text.is_some());
        let text = text.unwrap();
        assert!(!text.is_empty());
        assert!(text.contains("Lorem ipsum"));
        assert!(text.contains("dolor sit"));

        assert!(tool_calls.is_none());
        assert!(usage.is_none());
        assert!(errors.is_some()); // Lorem provider generates some test errors
        assert!(!cancelled);

        // End stream
        assert!(writer.end().await.is_ok());

        // Stream should be deleted after end
        assert!(!check_chat_stream_exists(&client, &user_id, &session_id)
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_stream_writer_batching() {
        let redis = setup_redis_pool().await;
        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let mut writer = create_test_writer(&redis, &user_id, &session_id).await;

        assert!(writer.start().await.is_ok());

        // Create a custom stream with small chunks to test batching
        let chunks = vec![
            "Hello", " ", "world", "!", " ", "This", " ", "is", " ", "a", " ", "test",
        ];
        let chunk_stream = tokio_stream::iter(
            chunks
                .into_iter()
                .map(|text| Ok(LlmStreamChunk::Text(text.into()))),
        );

        let stream: LlmStream = Box::pin(chunk_stream);
        let (text, _, _, _, cancelled) = writer.process(stream).await;

        assert!(text.is_some());
        let text = text.unwrap();
        assert_eq!(text, "Hello world! This is a test");
        assert!(!cancelled);

        writer.end().await.ok();
    }

    #[tokio::test]
    async fn test_stream_writer_error_handling() {
        let redis = setup_redis_pool().await;
        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let mut writer = create_test_writer(&redis, &user_id, &session_id).await;

        assert!(writer.start().await.is_ok());

        // Create a stream that produces an error
        let error_stream = tokio_stream::iter(vec![
            Ok(LlmStreamChunk::Text("Hello".to_string())),
            Err(LlmStreamError::ProviderError("Test error".into())),
            Ok(LlmStreamChunk::Text(" World".to_string())),
        ]);

        let stream: LlmStream = Box::pin(error_stream);
        let (text, _, _, errors, cancelled) = writer.process(stream).await;

        assert!(text.is_some());
        let text = text.unwrap();
        assert_eq!(text, "Hello World");

        assert!(errors.is_some());
        let errors = errors.unwrap();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("Test error")));

        assert!(!cancelled);

        writer.end().await.ok();
    }

    #[tokio::test]
    async fn test_stream_writer_timeout() {
        let redis = setup_redis_pool().await;
        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let mut writer = create_test_writer(&redis, &user_id, &session_id).await;

        assert!(writer.start().await.is_ok());

        // Create a stream that hangs (never yields anything)
        let hanging_stream = tokio_stream::pending::<Result<LlmStreamChunk, LlmStreamError>>();

        let stream: LlmStream = Box::pin(hanging_stream);

        // This should timeout due to LLM_TIMEOUT
        let start = std::time::Instant::now();
        let (text, _, _, errors, cancelled) = writer.process(stream).await;
        let elapsed = start.elapsed();

        // Should complete in roughly LLM_TIMEOUT duration
        assert!(elapsed >= Duration::from_secs(59)); // Allow some margin
        assert!(elapsed < Duration::from_secs(65));

        assert!(text.is_none());
        assert!(errors.is_some());
        let errors = errors.unwrap();
        assert!(errors.iter().any(|e| e.contains("Timeout")));
        assert!(!cancelled); // Timeout is not considered a cancellation

        writer.end().await.ok();
    }

    #[tokio::test]
    async fn test_stream_writer_cancel() {
        let redis = setup_redis_pool().await;
        let client = redis.get().await.unwrap();
        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let writer = create_test_writer(&redis, &user_id, &session_id).await;

        assert!(writer.start().await.is_ok());
        assert!(check_chat_stream_exists(&client, &user_id, &session_id)
            .await
            .unwrap());

        // Cancel the stream
        assert!(cancel_current_chat_stream(&client, &user_id, &session_id)
            .await
            .is_ok());

        // Stream should be deleted after cancel
        assert!(!check_chat_stream_exists(&client, &user_id, &session_id)
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_stream_writer_usage_tracking() {
        let redis = setup_redis_pool().await;
        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let mut writer = create_test_writer(&redis, &user_id, &session_id).await;

        assert!(writer.start().await.is_ok());

        // Create a stream with usage information
        let usage_stream = tokio_stream::iter(vec![
            Ok(LlmStreamChunk::Text("Hello".into())),
            Ok(LlmStreamChunk::Usage(LlmUsage {
                input_tokens: Some(10),
                output_tokens: Some(5),
                cost: Some(0.001),
            })),
            Ok(LlmStreamChunk::Text(" World".into())),
            Ok(LlmStreamChunk::Usage(LlmUsage {
                input_tokens: None,     // Should not override
                output_tokens: Some(7), // Should update
                cost: Some(0.002),      // Should update
            })),
        ]);

        let stream: LlmStream = Box::pin(usage_stream);
        let (text, _, usage, _, cancelled) = writer.process(stream).await;

        assert!(text.is_some());
        assert_eq!(text.unwrap(), "Hello World");

        assert!(usage.is_some());
        let usage = usage.unwrap();
        assert_eq!(usage.input_tokens, Some(10));
        assert_eq!(usage.output_tokens, Some(7));
        assert_eq!(usage.cost, Some(0.002));

        assert!(!cancelled);

        writer.end().await.ok();
    }

    #[tokio::test]
    async fn test_redis_stream_entries() {
        let redis = setup_redis_pool().await;
        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let mut writer = create_test_writer(&redis, &user_id, &session_id).await;
        let key = writer.key.clone();

        assert!(writer.start().await.is_ok());

        // Verify start event was written
        let entries: Vec<(String, HashMap<String, String>)> = redis
            .get()
            .await
            .expect("Failed to get Redis connection")
            .xrange(&key, "-", "+", None)
            .await
            .expect("Failed to read stream");

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].1.get("type"), Some(&"start".to_string()));

        // Create a simple stream
        let simple_stream = tokio_stream::iter(vec![Ok(LlmStreamChunk::Text("Test chunk".into()))]);
        let stream: LlmStream = Box::pin(simple_stream);
        writer.process(stream).await;
        writer.flush_chunk().await.ok();

        // Should have start + text entries (ping task may add more)
        let final_entries: Vec<(String, HashMap<String, String>)> = redis
            .get()
            .await
            .expect("Failed to get Redis connection")
            .xrange(&key, "-", "+", None)
            .await
            .expect("Failed to read stream");

        assert!(final_entries.len() >= 2);

        // Check that we have at least a text entry
        let has_text = final_entries
            .iter()
            .any(|(_, data)| data.get("type") == Some(&"text".to_string()));
        assert!(has_text);

        writer.end().await.ok();
    }
}
