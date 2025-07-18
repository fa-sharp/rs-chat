use chrono::{DateTime, Utc};
use diesel::{
    prelude::{Associations, Identifiable, Queryable},
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
    id: i32,
    name: String,
    pub provider_type: String,
    user_id: Uuid,
    pub base_url: Option<String>,
    api_key_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRsProviderType {
    Anthropic,
    Openai,
    Lorem,
}

impl TryFrom<&str> for ChatRsProviderType {
    type Error = LlmError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(value).map_err(|_| LlmError::UnsupportedProvider)
    }
}

impl TryFrom<ChatRsProviderType> for String {
    type Error = LlmError;

    fn try_from(value: ChatRsProviderType) -> Result<Self, Self::Error> {
        serde_json::to_string(&value).map_err(|_| LlmError::UnsupportedProvider)
    }
}
