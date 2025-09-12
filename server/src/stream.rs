mod llm_writer;
mod reader;
mod test_utils;

use std::{collections::HashMap, time::Duration};

use fred::{
    prelude::{FredResult, KeysInterface, StreamsInterface},
    types::scan::ScanType,
};

pub use llm_writer::*;
pub use reader::*;

use rocket::{
    async_trait,
    http::Status,
    request::{FromRequest, Outcome},
    Request,
};
use rocket_okapi::OpenApiFromRequest;
use uuid::Uuid;

/// Get the key prefix for the user's chat streams in Redis
fn get_chat_stream_prefix(user_id: &Uuid) -> String {
    format!("user:{}:chat:", user_id)
}

/// Get the key of the chat stream in Redis for the given user and session ID
fn get_chat_stream_key(user_id: &Uuid, session_id: &Uuid) -> String {
    format!("{}{}", get_chat_stream_prefix(user_id), session_id)
}

/// Get the ongoing chat stream sessions for a user.
pub async fn get_current_chat_streams(
    redis: &fred::clients::Client,
    user_id: &Uuid,
) -> FredResult<Vec<String>> {
    use fred::bytes_utils::Str;

    let prefix = get_chat_stream_prefix(user_id);
    let pattern = format!("{}*", prefix);
    let (_, keys): (Str, Vec<Str>) = redis
        .scan_page("0", &pattern, Some(20), Some(ScanType::Stream))
        .await?;

    // Get the last 2 entries in each stream to check if they are still active
    let pipeline = redis.pipeline();
    for key in &keys {
        let _: () = pipeline.xrevrange(key, "+", "-", Some(2)).await?;
    }
    let streams: Vec<Option<Vec<(Str, HashMap<Str, Str>)>>> = pipeline.all().await?;

    let active_keys = keys
        .into_iter()
        .zip(streams.into_iter())
        .filter_map(|(key, stream)| {
            // Filter out streams that have already ended or been cancelled
            if let Some(events) = stream {
                if events.iter().any(|(_, data)| {
                    data.get("type")
                        .is_some_and(|t| *t == "end" || *t == "cancel")
                }) {
                    return None;
                }
            }
            Some(key.strip_prefix(&prefix)?.to_string())
        })
        .collect();

    Ok(active_keys)
}

/// Check if the chat stream exists.
pub async fn check_chat_stream_exists(
    redis: &fred::clients::Client,
    user_id: &Uuid,
    session_id: &Uuid,
) -> FredResult<bool> {
    let key = get_chat_stream_key(user_id, session_id);
    let first_entry: Option<()> = redis.xread(Some(1), None, &key, "0-0").await?;
    Ok(first_entry.is_some())
}

/// Cancel a stream by adding a `cancel` event to the stream and then deleting it from Redis
pub async fn cancel_current_chat_stream(
    redis: &fred::clients::Client,
    user_id: &Uuid,
    session_id: &Uuid,
) -> FredResult<()> {
    let key = get_chat_stream_key(user_id, session_id);
    let entry: HashMap<String, String> = RedisStreamChunk::Cancel.into();
    let _: () = redis.xadd(&key, true, None, "*", entry).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;
    redis.del(&key).await
}

/// Request guard to extract the Last-Event-ID from the request headers
#[derive(OpenApiFromRequest)]
pub struct LastEventId(String);
impl std::ops::Deref for LastEventId {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[async_trait]
impl<'r> FromRequest<'r> for LastEventId {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.headers().get_one("Last-Event-ID") {
            Some(event_id) => Outcome::Success(LastEventId(event_id.to_owned())),
            None => Outcome::Error((Status::BadRequest, ())),
        }
    }
}
