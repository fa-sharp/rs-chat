#[cfg(test)]
mod tests;

mod tinistream;
mod writer;

pub use tinistream::*;
pub use writer::*;

use rocket::{
    async_trait,
    http::Status,
    request::{FromRequest, Outcome},
    Request,
};
use rocket_okapi::OpenApiFromRequest;
use uuid::Uuid;

/// Get the key prefix for the user's chat streams in Redis
pub fn chat_stream_prefix(user_id: &Uuid) -> String {
    format!("user:{}:chat:", user_id)
}

/// Get the key of the chat stream in Redis for the given user and session ID
pub fn chat_stream_key(user_id: &Uuid, session_id: &Uuid) -> String {
    format!("{}{}", chat_stream_prefix(user_id), session_id)
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
