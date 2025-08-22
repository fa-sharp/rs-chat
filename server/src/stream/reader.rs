use std::collections::HashMap;

use fred::{
    prelude::{KeysInterface, StreamsInterface},
    types::scan::ScanType,
};
use rocket::response::stream::Event;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    provider::LlmError,
    stream::{get_chat_stream_key, get_chat_stream_prefix},
};

/// Timeout for the blocking `xread` command.
const XREAD_BLOCK_TIMEOUT: u64 = 10_000; // 10 seconds

/// Utility for reading SSE events from a Redis stream.
pub struct SseStreamReader {
    redis: fred::prelude::Pool,
}

impl SseStreamReader {
    pub fn new(redis: &fred::prelude::Pool) -> Self {
        Self {
            redis: redis.clone(),
        }
    }

    /// Get the ongoing chat streams for a user.
    pub async fn get_chat_streams(&self, user_id: &Uuid) -> Result<Vec<String>, LlmError> {
        let pattern = format!("{}:*", get_chat_stream_prefix(user_id));
        let keys = self
            .redis
            .scan_page("0", &pattern, Some(20), Some(ScanType::Stream))
            .await?;
        Ok(keys)
    }

    /// Retrieve the previous events from the given Redis stream.
    /// Returns a tuple containing the previous events, the last event ID, and a boolean
    /// indicating if the stream has already ended.
    pub async fn get_prev_events(
        &self,
        user_id: &Uuid,
        session_id: &Uuid,
    ) -> Result<(Vec<Event>, String, bool), LlmError> {
        let key = get_chat_stream_key(user_id, session_id);
        let (_, prev_events): (String, Vec<(String, HashMap<String, String>)>) = self
            .redis
            .xread::<Option<Vec<_>>, _, _>(None, None, &key, "0-0")
            .await?
            .and_then(|mut streams| streams.pop()) // should only be 1 stream since we're sending 1 key in the command
            .ok_or(LlmError::StreamNotFound)?;
        let (last_event_id, is_end) = prev_events
            .last()
            .map(|(id, data)| (id.to_owned(), data.get("type").is_some_and(|t| t == "end")))
            .unwrap_or_else(|| ("0-0".into(), false));
        let sse_events = prev_events
            .into_iter()
            .map(convert_redis_event_to_sse)
            .collect::<Vec<_>>();

        Ok((sse_events, last_event_id, is_end))
    }

    /// Stream the events from the given Redis stream using a blocking `xread` command.
    pub async fn stream(
        &self,
        user_id: &Uuid,
        session_id: &Uuid,
        last_event_id: &str,
        tx: &mpsc::Sender<Event>,
    ) {
        let key = get_chat_stream_key(user_id, session_id);
        let mut last_event_id = last_event_id.to_owned();
        loop {
            match self.get_next_event(&key, &mut last_event_id, tx).await {
                Ok((id, data, is_end)) => {
                    let event = convert_redis_event_to_sse((id, data));
                    if let Err(_) = tx.send(event).await {
                        break; // client disconnected
                    }
                    if is_end {
                        break; // reached end of stream
                    }
                }
                Err(err) => {
                    let event = Event::data(format!("Error: {}", err)).event("error");
                    tx.send(event).await.ok();
                    break;
                }
            }
        }
    }

    /// Get the next event from the given Redis stream using a blocking `xread` command.
    /// - Updates the last event ID
    /// - Cancels waiting for the next event if the client disconnects
    /// - Returns the event ID, data, and a `bool` indicating whether it's the ending event
    async fn get_next_event(
        &self,
        key: &str,
        last_event_id: &mut String,
        tx: &mpsc::Sender<Event>,
    ) -> Result<(String, HashMap<String, String>, bool), LlmError> {
        let (_, mut events): (String, Vec<(String, HashMap<String, String>)>) = tokio::select! {
            res = self.redis.xread::<Option<Vec<_>>, _, _>(Some(1), Some(XREAD_BLOCK_TIMEOUT), key, &*last_event_id) => {
                match res?.as_mut().and_then(|streams| streams.pop()) {
                    Some(stream) => stream,
                    None => return Err(LlmError::StreamNotFound),
                }
            },
            _ = tx.closed() => return Err(LlmError::ClientDisconnected)
        };
        match events.pop() {
            Some((id, data)) => {
                *last_event_id = id.clone();
                let is_end = data.get("type").is_some_and(|t| t == "end");
                Ok((id, data, is_end))
            }
            None => Err(LlmError::NoStreamEvent),
        }
    }
}

/// Convert a Redis stream event into an SSE event. Expects the event hash map to contain
/// a "type" and "data" field (e.g. serialized using the appropriate serde tag and content).
fn convert_redis_event_to_sse((id, event): (String, HashMap<String, String>)) -> Event {
    let mut r#type: Option<String> = None;
    let mut data: Option<String> = None;
    for (key, value) in event {
        match key.as_str() {
            "type" => r#type = Some(value),
            "data" => data = Some(value),
            _ => {}
        }
    }

    Event::data(data.unwrap_or_default())
        .event(r#type.unwrap_or_else(|| "unknown".into()))
        .id(id)
}
