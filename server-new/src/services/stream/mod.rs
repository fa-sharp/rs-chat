use futures::{
    StreamExt,
    stream::{SplitSink, SplitStream},
};
use reqwest_websocket::WebSocket;
use tinistream::TinistreamClient;
use tinistream_client::types::{StreamAccessResponse, StreamInfo, StreamStatus};
use uuid::Uuid;

pub mod error;
pub mod tinistream;
mod writer;

#[cfg(test)]
mod tests;

use crate::{
    db::models::ChatRsLogStatus,
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
impl LlmStreamOutput {
    /// Get the logged status for this response
    pub fn status(&self) -> ChatRsLogStatus {
        if self.cancelled {
            ChatRsLogStatus::Cancelled
        } else if self.errors.is_some() {
            ChatRsLogStatus::Failed
        } else {
            ChatRsLogStatus::Completed
        }
    }
}

type WsWriter = SplitSink<WebSocket, reqwest_websocket::Message>;
type WsReader = SplitStream<WebSocket>;

impl<'r> StreamingService<'r> {
    pub fn new(tinistream: &'r TinistreamClient) -> Self {
        Self { tinistream }
    }

    /// Get the Redis key of the chat stream for the given user and session ID
    pub fn chat_stream_key(user_id: &Uuid, session_id: &Uuid) -> String {
        format!("{}{}", Self::chat_stream_prefix(user_id), session_id)
    }

    /// Get the Redis key prefix for the user's chat streams
    pub fn chat_stream_prefix(user_id: &Uuid) -> String {
        format!("user:{user_id}:chat:")
    }

    /// Generate a Redis key for a user's prompt
    pub fn prompt_key(user_id: &Uuid) -> String {
        format!("user:{user_id}:prompt:{}", Uuid::new_v4())
    }

    /// Extract the session ID from the user's stream key
    pub fn session_id_from_stream_key(key: &str, key_prefix: &str) -> Option<Uuid> {
        key.strip_prefix(key_prefix)
            .and_then(|session_id| Uuid::try_parse(session_id).ok())
    }

    /// Check for existing active client stream
    pub async fn exists_stream(&self, stream_key: &str) -> Result<bool, StreamingError> {
        Ok(self.tinistream.stream_exists(stream_key).await?)
    }

    /// Currently active streams with the given prefix
    pub async fn active_streams(&self, prefix: &str) -> Result<Vec<StreamInfo>, StreamingError> {
        let streams = self
            .tinistream
            .active_streams(&format!("{prefix}*",))
            .await?;

        Ok(streams)
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

    /// Get access to an ongoing client stream
    pub async fn access_stream(
        &self,
        stream_key: &str,
    ) -> Result<StreamAccessResponse, StreamingError> {
        Ok(self.tinistream.stream_connect(stream_key).await?)
    }

    /// Process and write the LLM response stream via the WebSocket connection,
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

    /// Signal stream cancellation
    pub async fn cancel_stream(&self, stream_key: &str) -> Result<StreamStatus, StreamingError> {
        Ok(self.tinistream.stream_cancel(stream_key).await?)
    }
}
