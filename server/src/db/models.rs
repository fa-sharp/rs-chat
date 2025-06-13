use chrono::{DateTime, Utc};
use diesel::{
    prelude::{Associations, Identifiable, Insertable, Queryable},
    Selectable,
};
use schemars::JsonSchema;
use uuid::Uuid;

#[derive(Identifiable, Queryable, Selectable, JsonSchema, serde::Serialize)]
#[diesel(table_name = super::schema::chat_sessions)]
pub struct ChatSession {
    pub id: Uuid,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::chat_sessions)]
pub struct NewChatSession<'r> {
    pub title: &'r str,
}

#[derive(diesel_derive_enum::DbEnum)]
#[db_enum(existing_type_path = "crate::db::schema::sql_types::ChatMessageRole")]
#[derive(Debug, JsonSchema, serde::Serialize)]
pub enum ChatMessageRole {
    User,
    Assistant,
    System,
}

#[derive(Identifiable, Queryable, Selectable, Associations, JsonSchema, serde::Serialize)]
#[diesel(belongs_to(ChatSession, foreign_key = session_id))]
#[diesel(table_name = super::schema::chat_messages)]
pub struct ChatMessage {
    pub id: Uuid,
    pub session_id: Uuid,
    pub role: ChatMessageRole,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::chat_messages)]
pub struct NewChatMessage<'r> {
    pub session_id: Uuid,
    pub role: ChatMessageRole,
    pub content: &'r str,
}
