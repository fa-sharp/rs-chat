use chrono::{DateTime, Utc};
use diesel::{
    prelude::{Associations, Identifiable, Insertable, Queryable},
    Selectable,
};
use schemars::JsonSchema;
use uuid::Uuid;

use crate::db::models::ChatRsUser;

#[derive(diesel_derive_enum::DbEnum)]
#[db_enum(existing_type_path = "crate::db::schema::sql_types::LlmProvider")]
#[derive(Debug, JsonSchema, serde::Serialize, serde::Deserialize)]
pub enum ChatRsProviderKeyType {
    Anthropic,
    Openai,
    Openrouter,
    // Ollama,
    // Deepseek,
    // Google,
}

#[derive(Identifiable, Queryable, Selectable, Associations)]
#[diesel(belongs_to(ChatRsUser, foreign_key = user_id))]
#[diesel(table_name = super::schema::secrets)]
pub struct ChatRsSecret {
    pub id: Uuid,
    pub user_id: Uuid,
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub created_at: DateTime<Utc>,
}

#[derive(Identifiable, Queryable, Selectable, Associations, JsonSchema, serde::Serialize)]
#[diesel(belongs_to(ChatRsUser, foreign_key = user_id))]
#[diesel(table_name = super::schema::secrets)]
pub struct ChatRsSecretMeta {
    pub id: Uuid,
    #[serde(skip)]
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::secrets)]
pub struct NewChatRsSecret<'r> {
    pub user_id: &'r Uuid,
    pub provider: &'r ChatRsProviderKeyType,
    pub ciphertext: &'r Vec<u8>,
    pub nonce: &'r Vec<u8>,
}
