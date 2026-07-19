use chrono::{DateTime, Utc};
use diesel::{deserialize::FromSqlRow, expression::AsExpression, prelude::*};
use diesel_jsonb_derive::AsJsonb;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::models::ChatRsUser;

#[derive(Identifiable, Associations, Queryable, Selectable, Serialize, JsonSchema)]
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

#[derive(Debug, Default, Serialize, Deserialize, JsonSchema, FromSqlRow, AsExpression, AsJsonb)]
#[diesel(sql_type = diesel::sql_types::Jsonb)]
pub struct ChatRsSessionMeta {
    // /// User configuration of tools for this session
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub tool_config: Option<SendChatToolInput>,
}
impl ChatRsSessionMeta {
    pub fn new() -> Self {
        Self {}
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
    pub meta: Option<ChatRsSessionMeta>,
}

#[derive(diesel_derive_enum::DbEnum)]
#[db_enum(existing_type_path = "crate::db::schema::sql_types::ChatMessageRole")]
#[derive(Debug, strum::EnumIs, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ChatRsMessageRole {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Identifiable, Queryable, Selectable, Associations, Serialize, JsonSchema)]
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

#[derive(Debug, Default, Serialize, Deserialize, JsonSchema, AsExpression, FromSqlRow, AsJsonb)]
#[diesel(sql_type = diesel::sql_types::Jsonb)]
pub struct ChatRsMessageMeta {
    /// User messages: metadata associated with the user message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserMeta>,
    /// Assistant messages: metadata associated with the assistant message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assistant: Option<AssistantMeta>,
    // /// Tool messages: metadata of the executed tool call
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub tool_call: Option<ChatRsExecutedToolCall>,
}
impl ChatRsMessageMeta {
    pub fn new_assistant(assistant_meta: AssistantMeta) -> Self {
        Self {
            assistant: Some(assistant_meta),
            ..Default::default()
        }
    }
    pub fn new_user(user_meta: UserMeta) -> Self {
        Self {
            user: Some(user_meta),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct UserMeta {
    /// The IDs of the files attached to this message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<Uuid>>,
}

#[derive(Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct AssistantMeta {
    // /// The tool calls requested by the assistant
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub tool_calls: Option<Vec<ChatRsToolCall>>,
    /// IDs of generated files
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<Uuid>>,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::chat_messages)]
pub struct NewChatRsMessage<'r> {
    pub session_id: &'r Uuid,
    pub role: ChatRsMessageRole,
    pub content: &'r str,
    pub meta: ChatRsMessageMeta,
}
