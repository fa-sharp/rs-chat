use chrono::{DateTime, Utc};
use diesel::{
    prelude::{AsChangeset, Associations, Identifiable, Insertable, Queryable},
    Selectable,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{db::models::ChatRsUser, provider::LlmError};

#[derive(Identifiable, Associations, Queryable, Selectable, JsonSchema, Serialize)]
#[diesel(belongs_to(ChatRsUser, foreign_key = user_id))]
#[diesel(table_name = super::schema::providers)]
pub struct ChatRsProvider {
    pub id: i32,
    pub name: String,
    #[schemars(with = "ChatRsProviderType")]
    pub provider_type: String,
    #[serde(skip_serializing)]
    pub user_id: Uuid,
    pub default_model: String,
    pub base_url: Option<String>,
    pub api_key_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::providers)]
pub struct NewChatRsProvider<'a> {
    pub name: &'a str,
    pub provider_type: &'a str,
    pub user_id: &'a Uuid,
    pub base_url: Option<&'a str>,
    pub default_model: &'a str,
    pub api_key_id: Option<Uuid>,
}

#[derive(Default, AsChangeset)]
#[diesel(table_name = super::schema::providers)]
pub struct UpdateChatRsProvider<'a> {
    pub name: Option<&'a str>,
    pub base_url: Option<&'a str>,
    pub default_model: Option<&'a str>,
    pub api_key_id: Option<Uuid>,
}

/// The API type of the provider
#[derive(JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatRsProviderType {
    Anthropic,
    Openai,
    Lorem,
}

impl TryFrom<&str> for ChatRsProviderType {
    type Error = LlmError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "anthropic" => Ok(ChatRsProviderType::Anthropic),
            "openai" => Ok(ChatRsProviderType::Openai),
            "lorem" => Ok(ChatRsProviderType::Lorem),
            _ => Err(LlmError::UnsupportedProvider),
        }
    }
}

impl From<&ChatRsProviderType> for &str {
    fn from(value: &ChatRsProviderType) -> Self {
        match value {
            ChatRsProviderType::Anthropic => "anthropic",
            ChatRsProviderType::Openai => "openai",
            ChatRsProviderType::Lorem => "lorem",
        }
    }
}
