use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use reqwest_websocket::WebSocket;
use rocket::futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt, TryStreamExt,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    db::models::ChatRsToolCall,
    provider::{LlmPendingToolCall, LlmStream, LlmStreamChunk, LlmStreamError, LlmUsage},
    stream::chat_stream_key,
};

/// Interval at which chunks are flushed to the Redis stream.
const FLUSH_INTERVAL: Duration = Duration::from_millis(500);
/// Max accumulated size of the text chunk before it is automatically flushed to Redis.
const MAX_CHUNK_SIZE: usize = 200;
/// Timeout waiting for data from the LLM stream.
const LLM_TIMEOUT: Duration = Duration::from_secs(60);

/// Utility for processing an incoming LLM response stream and writing to a Redis stream.
#[derive(Debug)]
pub struct LlmStreamWriter {
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
    /// WebSocket writer for writing to tinistream
    ws_writer: SplitSink<WebSocket, reqwest_websocket::Message>,
    /// WebSocket reader for reading responses from tinistream
    ws_reader: SplitStream<WebSocket>,
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
#[serde(tag = "event", content = "data", rename_all = "snake_case")]
pub(super) enum RedisStreamChunk {
    Text(String),
    ToolCall(String),
    PendingToolCall(String),
    Error(String),
}
impl From<RedisStreamChunk> for HashMap<String, String> {
    /// Converts a `RedisStreamChunk` into a hash map, suitable for the Redis client.
    fn from(chunk: RedisStreamChunk) -> Self {
        let value = serde_json::to_value(chunk).unwrap_or_default();
        serde_json::from_value(value).unwrap_or_default()
    }
}

impl LlmStreamWriter {
    pub fn new(
        ws_writer: SplitSink<WebSocket, reqwest_websocket::Message>,
        ws_reader: SplitStream<WebSocket>,
        user_id: &Uuid,
        session_id: &Uuid,
    ) -> Self {
        LlmStreamWriter {
            ws_writer,
            ws_reader,
            key: chat_stream_key(user_id, session_id),
            current_chunk: ChunkState::default(),
            complete_text: None,
            tool_calls: None,
            errors: None,
            usage: None,
        }
    }

    /// Key of the Redis stream.
    pub fn key(&self) -> &str {
        &self.key
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

        self.ws_writer.close().await.ok();

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
    pub(super) async fn flush_chunk(&mut self) -> Result<(), LlmStreamError> {
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
        self.add_to_stream(entries).await
    }

    /// Adds new entries to the Redis stream, while also checking for cancellation.
    /// Returns a [`LlmStreamError::StreamCancelled`] error if the stream has been cancelled.
    async fn add_to_stream(
        &mut self,
        entries: Vec<HashMap<String, String>>,
    ) -> Result<(), LlmStreamError> {
        use reqwest_websocket::Message;

        for entry in entries {
            let message = Message::text_from_json(&entry)?;
            self.ws_writer.send(message).await?;
            if let Some(response) = self.ws_reader.try_next().await? {
                if let Message::Close { .. } = response {
                    return Err(LlmStreamError::StreamCancelled);
                }
            }
        }

        Ok(())
    }
}
