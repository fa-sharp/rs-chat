//! Utilities for working with LLM requests and responses

use futures::TryStreamExt;
use serde::de::DeserializeOwned;
use tokio_stream::{Stream, StreamExt};
use tokio_util::{
    codec::{FramedRead, LinesCodec},
    io::StreamReader,
};

use crate::llm::error::LlmStreamChunkError;

/// Create a data URI
pub fn create_data_uri(content_type: &str, b64_string: &str) -> String {
    format!("data:{content_type};base64,{b64_string}")
}

/// Get a stream of deserialized events from a provider SSE stream.
pub fn get_sse_events<T: DeserializeOwned + Send + 'static>(
    response: reqwest::Response,
) -> impl Stream<Item = Result<T, LlmStreamChunkError>> {
    let stream_reader = StreamReader::new(response.bytes_stream().map_err(std::io::Error::other));
    let line_reader = FramedRead::new(stream_reader, LinesCodec::new());

    line_reader.filter_map(|line_result| {
        match line_result {
            Ok(line) => {
                if line.len() >= 6 && line.as_bytes().starts_with(b"data: ") {
                    let data = &line[6..]; // Skip "data: " prefix
                    if data.trim_start().is_empty() || data == "[DONE]" {
                        None // Skip empty lines and termination markers
                    } else {
                        Some(serde_json::from_str::<T>(data).map_err(LlmStreamChunkError::Parsing))
                    }
                } else {
                    None // Ignore non-data lines
                }
            }
            Err(e) => Some(Err(LlmStreamChunkError::Decoding(e))),
        }
    })
}

/// Get a stream of deserialized events from a provider JSON stream, not SSE (e.g. Ollama uses this format).
pub fn get_json_events<T: DeserializeOwned + Send + 'static>(
    response: reqwest::Response,
) -> impl Stream<Item = Result<T, LlmStreamChunkError>> {
    let stream_reader = StreamReader::new(response.bytes_stream().map_err(std::io::Error::other));
    let line_reader = FramedRead::new(stream_reader, LinesCodec::new());
    line_reader.map(|line_result| match line_result {
        Ok(line) => serde_json::from_str::<T>(&line).map_err(LlmStreamChunkError::Parsing),
        Err(e) => Err(LlmStreamChunkError::Decoding(e)),
    })
}
