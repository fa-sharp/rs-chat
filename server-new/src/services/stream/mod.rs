use futures::{
    StreamExt,
    stream::{SplitSink, SplitStream},
};
use reqwest_websocket::WebSocket;
use tinistream::TinistreamClient;
use tinistream_client::types::{StreamAccessResponse, StreamStatus};
use uuid::Uuid;

pub mod error;
pub mod tinistream;
mod writer;

#[cfg(test)]
mod tests;

use crate::{
    llm::{interface::LlmStream, types::LlmUsage},
    services::stream::error::StreamingError,
};

/// Handles stream processing and interacting with `tinistream` for streaming to users
pub struct StreamingService<'r> {
    tinistream: &'r TinistreamClient,
}

/// Complete, accumulated response from the LLM provider stream
pub struct LlmStreamOutput {
    pub text: Option<String>,
    // pub tool_calls: Option<Vec<ChatRsToolCall>>,
    // pub images: Option<Vec<LlmImage>>,
    pub usage: Option<LlmUsage>,
    pub errors: Option<Vec<String>>,
    pub cancelled: bool,
}

pub type WsWriter = SplitSink<WebSocket, reqwest_websocket::Message>;
pub type WsReader = SplitStream<WebSocket>;

impl<'r> StreamingService<'r> {
    pub fn new(tinistream: &'r TinistreamClient) -> Self {
        Self { tinistream }
    }

    /// Get the key of the chat stream in Redis for the given user and session ID
    pub fn chat_stream_key(user_id: &Uuid, session_id: &Uuid) -> String {
        format!("{}{}", Self::chat_stream_prefix(user_id), session_id)
    }

    /// Get the key prefix for the user's chat streams in Redis
    pub fn chat_stream_prefix(user_id: &Uuid) -> String {
        format!("user:{}:chat:", user_id)
    }

    /// Check for existing client stream
    pub async fn exists_stream(&self, stream_key: &str) -> Result<bool, StreamingError> {
        Ok(self.tinistream.stream_exists(&stream_key).await?)
    }

    /// Start the client stream, and return a WebSocket writer and reader for it
    pub async fn create_stream(
        &self,
        stream_key: &str,
    ) -> Result<(StreamAccessResponse, WsWriter, WsReader), StreamingError> {
        let stream_access = self.tinistream.stream_start(stream_key).await?;
        let (writer, reader) = self.tinistream.stream_writer_ws(stream_key).await?.split();

        Ok((stream_access, writer, reader))
    }

    /// Process and write the LLM stream via the WebSocket connection,
    /// and return the accumulated response.
    pub async fn process_stream(
        stream: LlmStream,
        writer: WsWriter,
        reader: WsReader,
    ) -> LlmStreamOutput {
        writer::LlmStreamWriter::new()
            .process(stream, writer, reader)
            .await
    }

    /// Signal end of stream
    pub async fn end_stream(&self, stream_key: &str) -> Result<StreamStatus, StreamingError> {
        Ok(self.tinistream.stream_end(stream_key).await?)
    }
}
