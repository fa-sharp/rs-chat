use chrono::{DateTime, Utc};
use diesel::prelude::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::{EnumString, IntoStaticStr};
use uuid::Uuid;

use crate::db::models::ChatRsUser;

#[derive(Identifiable, Associations, Queryable, Selectable, Serialize, JsonSchema)]
#[diesel(belongs_to(ChatRsUser, foreign_key = user_id))]
#[diesel(table_name = super::schema::providers)]
pub struct ChatRsProvider {
    pub id: i32,
    pub name: String,
    #[schemars(with = "ChatRsProviderType")]
    pub provider_type: String,
    #[schemars(with = "OpenAISubtype")]
    pub openai_subtype: Option<String>,
    #[serde(skip)]
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
    pub openai_subtype: Option<&'a str>,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ChatRsProviderType {
    /// OpenAI or OpenAI-compatible provider
    OpenAI,
    /// Anthropic provider
    Anthropic,
    /// Ollama provider
    Ollama,
    /// Lorem ipsum provider (for testing)
    Lorem,
}

/// The subtype for OpenAI-compatible providers
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr, Deserialize, JsonSchema,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum OpenAISubtype {
    #[default]
    OpenAI,
    OpenRouter,
}
