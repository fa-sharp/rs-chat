use chrono::{DateTime, Utc};
use diesel::{
    prelude::{AsChangeset, Associations, Identifiable, Insertable, Queryable},
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

#[derive(diesel_derive_enum::DbEnum)]
#[db_enum(existing_type_path = "crate::db::schema::sql_types::LlmProvider")]
#[derive(Debug, JsonSchema, serde::Serialize, serde::Deserialize)]
pub enum ChatRsApiKeyProviderType {
    Anthropic,
    Openai,
    Ollama,
    Deepseek,
    Google,
    Openrouter,
}

#[derive(Identifiable, Queryable, Selectable, Associations, JsonSchema, serde::Serialize)]
#[diesel(belongs_to(ChatRsUser, foreign_key = user_id))]
#[diesel(table_name = super::schema::api_keys)]
pub struct ChatRsApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: ChatRsApiKeyProviderType,
    #[serde(skip)]
    pub ciphertext: Vec<u8>,
    #[serde(skip)]
    pub nonce: Vec<u8>,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::api_keys)]
pub struct NewChatRsApiKey<'r> {
    pub user_id: &'r Uuid,
    pub provider: &'r ChatRsApiKeyProviderType,
    pub ciphertext: &'r Vec<u8>,
    pub nonce: &'r Vec<u8>,
}
