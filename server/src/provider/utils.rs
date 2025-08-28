use rocket::futures::TryStreamExt;
use serde::de::DeserializeOwned;
use tokio_stream::{Stream, StreamExt};
use tokio_util::{
    codec::{FramedRead, LinesCodec},
    io::StreamReader,
};

use crate::provider::LlmStreamError;

/// Get a stream of deserialized events from a provider SSE stream.
pub fn get_sse_events<T: DeserializeOwned + Send + 'static>(
    response: reqwest::Response,
) -> impl Stream<Item = Result<T, LlmStreamError>> {
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
                        Some(serde_json::from_str::<T>(data).map_err(LlmStreamError::Parsing))
                    }
                } else {
                    None // Ignore non-data lines
                }
            }
            Err(e) => Some(Err(LlmStreamError::Decoding(e))),
        }
    })
}
