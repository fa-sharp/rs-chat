use std::time::{Duration, Instant};

use futures::{SinkExt, StreamExt};
use reqwest_websocket::Message as WsMessage;
use serde::Serialize;
use tokio_util::sync::CancellationToken;

use crate::{
    llm::{
        error::LlmStreamChunkError,
        interface::{LlmStream, LlmStreamChunk},
        types::LlmUsage,
    },
    services::stream::{LlmStreamOutput, WsReader, WsWriter, error::StreamingError},
};

/// Interval at which chunks are flushed to the Redis stream.
const FLUSH_INTERVAL: Duration = Duration::from_millis(400);
/// Max # of characters of the text chunk before it is automatically flushed to Redis.
const MAX_CHUNK_SIZE: usize = 75;

/// Utility for processing an incoming LLM response stream and writing chunks to `tinistream`.
#[derive(Debug)]
pub struct LlmStreamWriter {
    /// The current chunk of data being processed.
    current_chunk: ChunkState,
    /// Accumulated text response from the assistant.
    complete_text: Option<String>,
    /// Accumulated tool calls from the assistant.
    // tool_calls: Option<Vec<ChatRsToolCall>>,
    /// Accumulated generated images from the assistant.
    // images: Option<Vec<LlmImage>>,
    /// Accumulated errors during the stream from the LLM provider.
    errors: Option<Vec<LlmStreamChunkError>>,
    /// Accumulated usage information from the LLM provider.
    usage: Option<LlmUsage>,
}

/// Internal state
#[derive(Debug, Default)]
struct ChunkState {
    text: Option<String>,
    // tool_calls: Option<Vec<ChatRsToolCall>>,
    // pending_tool_calls: Option<Vec<LlmPendingToolCall>>,
    error: Option<String>,
}

/// Chunk of the LLM response stored in the Redis stream.
#[derive(Debug, Serialize)]
#[serde(tag = "event", content = "data", rename_all = "snake_case")]
pub(super) enum RedisStreamChunk {
    Text(String),
    // ToolCall(String),
    // PendingToolCall(String),
    Error(String),
}

impl LlmStreamWriter {
    pub fn new() -> Self {
        LlmStreamWriter {
            current_chunk: ChunkState::default(),
            complete_text: None,
            // tool_calls: None,
            // images: None,
            errors: None,
            usage: None,
        }
    }

    /// Process the incoming stream from the LLM provider, intermittently flushing
    /// chunks to `tinistream` via the WebSocket connection, and return the final
    /// accumulated response.
    pub async fn process(
        &mut self,
        stream: LlmStream,
        mut writer: WsWriter,
        mut reader: WsReader,
    ) -> LlmStreamOutput {
        let mut cancelled = false;

        // Spawn task to listen for stream cancellation
        let cancel_token = CancellationToken::new();
        let cancel_task_token = cancel_token.clone();
        let cancel_task = tokio::spawn(async move {
            while let Some(res) = reader.next().await {
                if let Ok(WsMessage::Close { .. }) = res {
                    cancel_task_token.cancel();
                }
            }
        });

        tokio::select! {
            _ = self.process_stream(stream, &mut writer) => {}
            _ = cancel_token.cancelled() => {
                self.errors.get_or_insert_default().push(LlmStreamChunkError::StreamCancelled);
                cancelled = true;
            }
        }

        cancel_task.abort();
        writer.close().await.ok();

        LlmStreamOutput {
            text: self.complete_text.take(),
            // tool_calls: self.tool_calls.take(),
            // images: self.images.take(),
            usage: self.usage.take(),
            errors: self.errors.take().map(|e| {
                e.into_iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>()
            }),
            cancelled,
        }
    }

    async fn process_stream(&mut self, mut stream: LlmStream, writer: &mut WsWriter) {
        let mut last_flush_time = Instant::now();
        loop {
            match stream.next().await {
                Some(Ok(chunk)) => match chunk {
                    LlmStreamChunk::Text(text) => self.process_text(&text),
                    // LlmStreamChunk::ToolCalls(tool_calls) => self.process_tool_calls(tool_calls),
                    // LlmStreamChunk::PendingToolCall(pending_tool_call) => {
                    //     self.process_pending_tool_call(pending_tool_call)
                    // }
                    // LlmStreamChunk::Images(images) => self.process_images(images),
                    LlmStreamChunk::Usage(usage) => self.process_usage(usage),
                },
                Some(Err(err)) => self.process_error(err),
                None => break,
            }

            if self.should_flush(&last_flush_time) {
                if let Err(err) = self.flush_chunks(writer).await {
                    self.process_error(LlmStreamChunkError::from(err));
                }
                last_flush_time = Instant::now();
            }
        }

        if let Err(err) = self.flush_chunks(writer).await {
            self.process_error(LlmStreamChunkError::from(err));
        }
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

    // fn process_tool_calls(&mut self, tool_calls: Vec<ChatRsToolCall>) {
    //     self.current_chunk
    //         .tool_calls
    //         .get_or_insert_default()
    //         .extend(tool_calls.clone());
    //     self.tool_calls.get_or_insert_default().extend(tool_calls);
    // }

    // fn process_pending_tool_call(&mut self, tool_call: LlmPendingToolCall) {
    //     let current_chunk = self
    //         .current_chunk
    //         .pending_tool_calls
    //         .get_or_insert_default();
    //     if !current_chunk.iter().any(|tc| tc.index == tool_call.index) {
    //         current_chunk.push(tool_call);
    //     }
    // }

    // fn process_images(&mut self, images: Vec<LlmImage>) {
    //     self.images.get_or_insert_default().extend(images);
    // }

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

    fn process_error(&mut self, err: LlmStreamChunkError) {
        self.current_chunk.error = Some(err.to_string());
        self.errors.get_or_insert_default().push(err);
    }

    fn should_flush(&self, last_flush_time: &Instant) -> bool {
        // if self.current_chunk.tool_calls.is_some() || self.current_chunk.error.is_some() {
        //     return true;
        // }
        if self.current_chunk.error.is_some() {
            return true;
        }
        let text = self.current_chunk.text.as_ref();
        last_flush_time.elapsed() > FLUSH_INTERVAL || text.is_some_and(|t| t.len() > MAX_CHUNK_SIZE)
    }

    /// Flushes the current chunk(s) to the Redis stream.
    pub(super) async fn flush_chunks(
        &mut self,
        ws_writer: &mut WsWriter,
    ) -> Result<(), StreamingError> {
        let chunk_state = std::mem::take(&mut self.current_chunk);

        if let Some(text) = chunk_state.text {
            self.add_to_stream(ws_writer, RedisStreamChunk::Text(text))
                .await?;
        }
        // if let Some(tool_calls) = chunk_state.tool_calls {
        //     for tool_call in tool_calls {
        //         let tool_call_str = serde_json::to_string(&tool_call).unwrap_or_default();
        //         let entry = RedisStreamChunk::ToolCall(tool_call_str);
        //         self.add_to_stream(ws_writer, entry).await?;
        //     }
        // }
        // if let Some(pending_tool_calls) = chunk_state.pending_tool_calls {
        //     for tool_call in pending_tool_calls {
        //         let tool_call_str = serde_json::to_string(&tool_call).unwrap_or_default();
        //         let entry = RedisStreamChunk::PendingToolCall(tool_call_str);
        //         self.add_to_stream(ws_writer, entry).await?;
        //     }
        // }
        if let Some(error) = chunk_state.error {
            self.add_to_stream(ws_writer, RedisStreamChunk::Error(error))
                .await?;
        }

        Ok(ws_writer.flush().await?)
    }

    /// Serialize and add an entry to Redis via the WebSocket connection (does not flush the connection)
    async fn add_to_stream(
        &mut self,
        ws_writer: &mut WsWriter,
        entry: RedisStreamChunk,
    ) -> Result<(), StreamingError> {
        let message = WsMessage::text_from_json(&entry)?;
        Ok(ws_writer.feed(message).await?)
    }
}
