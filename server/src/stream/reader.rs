use std::collections::HashMap;

use fred::prelude::StreamsInterface;
use rocket::response::stream::Event;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{provider::LlmError, redis::ExclusiveRedisClient, stream::get_chat_stream_key};

/// Timeout in milliseconds for the blocking `xread` command.
const XREAD_BLOCK_TIMEOUT: u64 = 10_000; // 10 seconds

/// Utility for reading SSE events from a Redis stream.
pub struct SseStreamReader {
    redis: ExclusiveRedisClient,
}

impl SseStreamReader {
    pub fn new(redis: ExclusiveRedisClient) -> Self {
        Self { redis }
    }

    /// Retrieve the previous events from the given Redis stream.
    /// Returns a tuple containing the previous events, the last event ID, and a boolean
    /// indicating if the stream has already ended.
    pub async fn get_prev_events(
        &self,
        user_id: &Uuid,
        session_id: &Uuid,
        start_event_id: Option<&str>,
    ) -> Result<(Vec<Event>, String, bool), LlmError> {
        let key = get_chat_stream_key(user_id, session_id);
        let start_event_id = start_event_id.unwrap_or("0-0");
        let prev_events = self.xread(&key, start_event_id, None, None).await?;

        let (last_event_id, is_end) = prev_events
            .last()
            .map(|(id, data)| (id.to_owned(), is_end_event(&data)))
            .unwrap_or_else(|| (start_event_id.into(), false));
        let sse_events = prev_events
            .into_iter()
            .map(convert_redis_event_to_sse)
            .collect::<Vec<_>>();

        Ok((sse_events, last_event_id, is_end))
    }

    /// Stream SSE events from the given Redis stream using a blocking `xread` command.
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
            match self.next_event(&key, &mut last_event_id).await {
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

    /// Wait for the next event from the given Redis stream using a blocking `xread` command.
    /// - Cancels waiting for the next event upon the blocking timeout
    /// - Updates the last event ID with the ID of the received event
    /// - Returns the event ID, data, and a `bool` indicating whether it's an ending event
    async fn next_event(
        &self,
        key: &str,
        last_event_id: &mut String,
    ) -> Result<(String, HashMap<String, String>, bool), LlmError> {
        let (id, data) = self
            .xread(key, last_event_id, Some(1), Some(XREAD_BLOCK_TIMEOUT))
            .await?
            .pop() // only reading 1 event
            .ok_or(LlmError::NoStreamEvent)?;
        *last_event_id = id.clone();
        let is_end = is_end_event(&data);
        Ok((id, data, is_end))
    }

    /// Read events from the given stream (friendly `XREAD` wrapper that takes care of the
    /// weird types). Returns `LlmError::StreamNotFound` if there is no stream.
    async fn xread(
        &self,
        key: &str,
        start_event_id: &str,
        count: Option<u64>,
        block: Option<u64>,
    ) -> Result<Vec<(String, HashMap<String, String>)>, LlmError> {
        let (_key, events) = self
            .redis
            .xread::<Option<Vec<(String, _)>>, _, _>(count, block, key, start_event_id)
            .await?
            .and_then(|mut streams| streams.pop()) // should only be 1 stream since we're sending 1 key in the command
            .ok_or(LlmError::StreamNotFound)?;
        Ok(events)
    }
}

/// Check if this event is an ending event.
fn is_end_event(data: &HashMap<String, String>) -> bool {
    data.get("type")
        .is_some_and(|t| t == "end" || t == "cancel")
}

/// Convert a Redis stream event into an SSE event. Expects the event hash map to contain
/// a "type" and "data" field (e.g. serialized using the appropriate serde tag and content).
fn convert_redis_event_to_sse((id, hash): (String, HashMap<String, String>)) -> Event {
    let mut event: Option<String> = None;
    let mut data: Option<String> = None;
    for (key, value) in hash {
        match key.as_str() {
            "type" => event = Some(value),
            "data" => data = Some(format!(" {value}")), // SSE spec: add space before data
            _ => {}
        }
    }

    Event::data(data.unwrap_or_default())
        .event(event.unwrap_or_else(|| "unknown".into()))
        .id(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream::test_utils::setup_redis_pool;
    use fred::prelude::KeysInterface;
    use rand::distr::{Alphanumeric, SampleString};

    type RedisEvent = (String, HashMap<String, String>);

    #[tokio::test]
    async fn xread() -> Result<(), LlmError> {
        let redis = setup_redis_pool().await;
        let client = redis.get().await.expect("should get a client");
        let reader_client = redis.get().await.expect("should get a client");
        let reader = SseStreamReader::new(ExclusiveRedisClient(reader_client));

        let stream_key = format!(
            "stream_{}",
            Alphanumeric.sample_string(&mut rand::rng(), 10)
        );

        let event_1: RedisEvent = (
            "1".into(),
            HashMap::from([
                ("type".into(), "message".into()),
                ("data".into(), "Hello, world!".into()),
            ]),
        );
        let event_2: RedisEvent = (
            "2".into(),
            HashMap::from([
                ("type".into(), "message".into()),
                ("data".into(), "Goodbye, world!".into()),
            ]),
        );
        let event_3: RedisEvent = ("3".into(), HashMap::from([("type".into(), "end".into())]));
        for (id, data) in [event_1, event_2, event_3] {
            let _: () = client
                .xadd(&stream_key, false, None, id, data)
                .await
                .expect("should add event to Redis stream");
        }

        let mut event_1 = reader.xread(&stream_key, "0-0", Some(1), None).await?;
        assert_eq!(event_1.len(), 1);
        let (event_1_id, event_1_data) = event_1.pop().unwrap();
        assert_eq!(event_1_id, "1-0");
        assert_eq!(event_1_data["type"], "message");
        assert_eq!(event_1_data["data"], "Hello, world!");

        let mut event_2 = reader
            .xread(&stream_key, &event_1_id, Some(1), None)
            .await?;
        assert_eq!(event_2.len(), 1);
        let (event_2_id, event_2_data) = event_2.pop().unwrap();
        assert_eq!(event_2_id, "2-0");
        assert_eq!(event_2_data["type"], "message");
        assert_eq!(event_2_data["data"], "Goodbye, world!");

        let mut event_3 = reader
            .xread(&stream_key, &event_2_id, Some(1), None)
            .await?;
        assert_eq!(event_3.len(), 1);
        let (event_3_id, event_3_data) = event_3.pop().unwrap();
        assert_eq!(event_3_id, "3-0");
        assert_eq!(event_3_data["type"], "end");

        let _: () = client.del(&stream_key).await?;

        Ok(())
    }
}
