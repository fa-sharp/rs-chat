use chrono::{DateTime, Utc};
use diesel::{
    prelude::{AsChangeset, Associations, Identifiable, Insertable, Queryable},
    Selectable,
};
use diesel_as_jsonb::AsJsonb;
use rocket_okapi::OpenApiFromRequest;
use schemars::JsonSchema;
use uuid::Uuid;

use crate::utils::create_provider::ProviderConfigInput;

#[derive(Identifiable, Queryable, Selectable, JsonSchema, OpenApiFromRequest, serde::Serialize)]
#[diesel(table_name = super::schema::users)]
pub struct ChatRsUser {
    pub id: Uuid,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discord_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oidc_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sso_username: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Default)]
#[diesel(table_name = super::schema::users)]
pub struct NewChatRsUser<'r> {
    pub github_id: Option<&'r str>,
    pub google_id: Option<&'r str>,
    pub discord_id: Option<&'r str>,
    pub oidc_id: Option<&'r str>,
    pub sso_username: Option<&'r str>,
    pub name: &'r str,
    pub avatar_url: Option<&'r str>,
}

#[derive(AsChangeset, Default)]
#[diesel(table_name = super::schema::users)]
pub struct UpdateChatRsUser<'r> {
    pub github_id: Option<&'r str>,
    pub google_id: Option<&'r str>,
    pub discord_id: Option<&'r str>,
    pub oidc_id: Option<&'r str>,
    pub sso_username: Option<&'r str>,
    pub name: Option<&'r str>,
    pub avatar_url: Option<&'r str>,
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
    pub meta: ChatRsMessageMeta,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Default, JsonSchema, serde::Serialize, serde::Deserialize, AsJsonb)]
pub struct ChatRsMessageMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_config: Option<ProviderConfigInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interrupted: Option<bool>,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::chat_messages)]
pub struct NewChatRsMessage<'r> {
    pub session_id: &'r Uuid,
    pub role: ChatRsMessageRole,
    pub content: &'r str,
    pub meta: &'r ChatRsMessageMeta,
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
