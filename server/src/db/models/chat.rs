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
    tools::SendChatToolInput,
};

#[derive(Identifiable, Associations, Queryable, Selectable, JsonSchema, serde::Serialize)]
#[diesel(belongs_to(ChatRsUser, foreign_key = user_id))]
#[diesel(table_name = super::schema::chat_sessions)]
pub struct ChatRsSession {
    pub id: Uuid,
    #[serde(skip)]
    pub user_id: Uuid,
    pub title: String,
    pub meta: ChatRsSessionMeta,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Default, JsonSchema, Serialize, Deserialize, AsJsonb)]
pub struct ChatRsSessionMeta {
    /// User configuration of tools for this session
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_config: Option<SendChatToolInput>,
}
impl ChatRsSessionMeta {
    pub fn with_tool_config(tool_config: Option<SendChatToolInput>) -> Self {
        Self { tool_config }
    }
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::chat_sessions)]
pub struct NewChatRsSession<'r> {
    pub user_id: &'r Uuid,
    pub title: &'r str,
}

#[derive(AsChangeset, Default)]
#[diesel(table_name = super::schema::chat_sessions)]
pub struct UpdateChatRsSession<'r> {
    pub title: Option<&'r str>,
    pub meta: Option<&'r ChatRsSessionMeta>,
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
    /// Assistant messages: metadata associated with the assistant message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assistant: Option<AssistantMeta>,
    /// Tool messages: metadata of the executed tool call
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call: Option<ChatRsExecutedToolCall>,
}

#[derive(Debug, Default, JsonSchema, Serialize, Deserialize)]
pub struct AssistantMeta {
    /// The ID of the LLM provider used to generate this message
    pub provider_id: i32,
    /// Options passed to the LLM provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_options: Option<LlmApiProviderSharedOptions>,
    /// The tool calls requested by the assistant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ChatRsToolCall>>,
    /// Provider usage information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<LlmUsage>,
    /// Whether this is a partial and/or interrupted message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial: Option<bool>,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::chat_messages)]
pub struct NewChatRsMessage<'r> {
    pub session_id: &'r Uuid,
    pub role: ChatRsMessageRole,
    pub content: &'r str,
    pub meta: ChatRsMessageMeta,
}
