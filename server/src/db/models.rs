use chrono::{DateTime, Utc};
use diesel::{
    prelude::{Associations, Identifiable, Insertable, Queryable},
    Selectable,
};
use rocket_okapi::OpenApiFromRequest;
use schemars::JsonSchema;
use uuid::Uuid;

#[derive(Identifiable, Queryable, Selectable, JsonSchema, OpenApiFromRequest, serde::Serialize)]
#[diesel(table_name = super::schema::users)]
pub struct ChatRsUser {
    pub id: Uuid,
    pub github_id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::users)]
pub struct NewChatRsUser<'r> {
    pub github_id: &'r str,
    pub name: &'r str,
}

#[derive(Identifiable, Queryable, Selectable, JsonSchema, serde::Serialize)]
#[diesel(belongs_to(ChatRsUser, foreign_key = user_id))]
#[diesel(table_name = super::schema::chat_sessions)]
pub struct ChatRsSession {
    pub id: Uuid,
    #[serde(skip_serializing)]
    pub user_id: Uuid,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::chat_sessions)]
pub struct NewChatRsSession<'r> {
    pub title: &'r str,
}

#[derive(diesel_derive_enum::DbEnum)]
#[db_enum(existing_type_path = "crate::db::schema::sql_types::ChatMessageRole")]
#[derive(Debug, JsonSchema, serde::Serialize)]
pub enum ChatRsMessageRole {
    User,
    Assistant,
    System,
}

#[derive(Identifiable, Queryable, Selectable, Associations, JsonSchema, serde::Serialize)]
#[diesel(belongs_to(ChatRsSession, foreign_key = session_id))]
#[diesel(table_name = super::schema::chat_messages)]
pub struct ChatRsMessage {
    pub id: Uuid,
    #[serde(skip_serializing)]
    pub session_id: Uuid,
    pub role: ChatRsMessageRole,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::chat_messages)]
pub struct NewChatRsMessage<'r> {
    pub session_id: &'r Uuid,
    pub role: ChatRsMessageRole,
    pub content: &'r str,
}
