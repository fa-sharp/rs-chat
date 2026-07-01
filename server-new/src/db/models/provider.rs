use std::str::FromStr;

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::models::ChatRsUser;

#[derive(Identifiable, Associations, Queryable, Selectable, Serialize)]
#[diesel(belongs_to(ChatRsUser, foreign_key = user_id))]
#[diesel(table_name = super::schema::providers)]
pub struct ChatRsProvider {
    pub id: i32,
    pub name: String,
    // #[schemars(with = "ChatRsProviderType")]
    pub provider_type: String,
    // #[schemars(with = "OpenaiSubtype")]
    // pub openai_subtype: Option<String>,
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
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatRsProviderType {
    Anthropic,
    Openai,
    Ollama,
    Lorem,
}

/// The subtype for OpenAI-compatible providers
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OpenaiSubtype {
    Openai,
    Google,
    OpenRouter,
    LlmGateway,
}

#[derive(Debug, thiserror::Error)]
#[error("invalid provider type: '{0}'")]
pub struct ParseProviderTypeError(String);

impl FromStr for ChatRsProviderType {
    type Err = ParseProviderTypeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "anthropic" => Ok(ChatRsProviderType::Anthropic),
            "openai" => Ok(ChatRsProviderType::Openai),
            "ollama" => Ok(ChatRsProviderType::Ollama),
            "lorem" => Ok(ChatRsProviderType::Lorem),
            provider => Err(ParseProviderTypeError(provider.into())),
        }
    }
}

impl From<&ChatRsProviderType> for &str {
    fn from(value: &ChatRsProviderType) -> Self {
        match value {
            ChatRsProviderType::Anthropic => "anthropic",
            ChatRsProviderType::Openai => "openai",
            ChatRsProviderType::Ollama => "ollama",
            ChatRsProviderType::Lorem => "lorem",
        }
    }
}
