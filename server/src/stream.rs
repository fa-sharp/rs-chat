mod llm_writer;
mod reader;

pub use llm_writer::*;
pub use reader::*;

use uuid::Uuid;

/// Get the key prefix for the user's chat streams in Redis
fn get_chat_stream_prefix(user_id: &Uuid) -> String {
    format!("user:{}:chat", user_id)
}

/// Get the key of the chat stream in Redis for the given user and session ID
fn get_chat_stream_key(user_id: &Uuid, session_id: &Uuid) -> String {
    format!("{}:{}", get_chat_stream_prefix(user_id), session_id)
}
