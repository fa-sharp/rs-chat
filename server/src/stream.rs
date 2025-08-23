mod llm_writer;
mod reader;

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
    format!("user:{}:chat", user_id)
}

/// Get the key of the chat stream in Redis for the given user and session ID
fn get_chat_stream_key(user_id: &Uuid, session_id: &Uuid) -> String {
    format!("{}:{}", get_chat_stream_prefix(user_id), session_id)
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
