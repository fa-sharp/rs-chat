use chrono::{DateTime, Utc};
use diesel::{
    prelude::{AsChangeset, Associations, Identifiable, Insertable, Queryable},
    Selectable,
};
use diesel_as_jsonb::AsJsonb;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    db::models::{ChatRsExecutedToolCall, ChatRsToolCall, ChatRsUser},
    provider::{LlmApiProviderSharedOptions, LlmUsage},
};

#[derive(Identifiable, Associations, Queryable, Selectable, JsonSchema, serde::Serialize)]
#[diesel(belongs_to(ChatRsUser, foreign_key = user_id))]
#[diesel(table_name = super::schema::chat_sessions)]
pub struct ChatRsSession {
    pub id: Uuid,
    #[serde(skip)]
    pub user_id: Uuid,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::chat_sessions)]
pub struct NewChatRsSession<'r> {
    pub user_id: &'r Uuid,
    pub title: &'r str,
}

#[derive(AsChangeset)]
#[diesel(table_name = super::schema::chat_sessions)]
pub struct UpdateChatRsSession<'r> {
    pub title: &'r str,
}

#[derive(diesel_derive_enum::DbEnum)]
#[db_enum(existing_type_path = "crate::db::schema::sql_types::ChatMessageRole")]
#[derive(Debug, PartialEq, Eq, JsonSchema, serde::Serialize)]
pub enum ChatRsMessageRole {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Identifiable, Queryable, Selectable, Associations, JsonSchema, serde::Serialize)]
#[diesel(belongs_to(ChatRsSession, foreign_key = session_id))]
#[diesel(table_name = super::schema::chat_messages)]
pub struct ChatRsMessage {
    pub id: Uuid,
    pub session_id: Uuid,
    pub role: ChatRsMessageRole,
    pub content: String,
    pub meta: ChatRsMessageMeta,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Default, JsonSchema, Serialize, Deserialize, AsJsonb)]
pub struct ChatRsMessageMeta {
    /// Assistant messages: the tool calls requested by the assistant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ChatRsToolCall>>,
    /// Tool messages: the executed tool call that produced this result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executed_tool_call: Option<ChatRsExecutedToolCall>,
    /// Assistant messages: provider usage information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<LlmUsage>,
    /// Assistant messages: whether this is a partial or interrupted message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interrupted: Option<bool>,
    /// Assistant messages: options passed to the LLM provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_options: Option<LlmApiProviderSharedOptions>,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::chat_messages)]
pub struct NewChatRsMessage<'r> {
    pub session_id: &'r Uuid,
    pub role: ChatRsMessageRole,
    pub content: &'r str,
    pub meta: ChatRsMessageMeta,
}
