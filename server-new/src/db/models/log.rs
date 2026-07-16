use bigdecimal::BigDecimal;
use diesel::{deserialize::FromSqlRow, expression::AsExpression, prelude::*};
use diesel_jsonb_derive::AsJsonb;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum::{AsRefStr, EnumString};
use uuid::Uuid;

use crate::db::{
    UtcDateTime,
    models::{ChatRsMessage, ChatRsUser},
};

#[derive(Identifiable, Associations, Queryable, Selectable, AsChangeset)]
#[diesel(belongs_to(ChatRsUser, foreign_key = user_id))]
#[diesel(belongs_to(ChatRsMessage, foreign_key = message_id))]
#[diesel(table_name = super::schema::llm_logs)]
pub struct ChatRsLog {
    pub id: i32,
    pub kind: String,
    pub user_id: Uuid,
    pub provider_id: Option<i32>,
    pub session_id: Option<Uuid>,
    pub message_id: Option<Uuid>,
    pub model: String,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub cost: Option<BigDecimal>,
    pub ttft_ms: Option<i32>,
    pub status: String,
    pub meta: ChatRsLogMeta,
    pub started_at: UtcDateTime,
    pub completed_at: Option<UtcDateTime>,
}

#[skip_serializing_none]
#[derive(Identifiable, Queryable, Selectable, Serialize, JsonSchema)]
#[diesel(table_name = super::schema::llm_logs)]
pub struct ChatRsLogLlmRequest {
    #[serde(skip)]
    pub id: i32,
    pub provider_id: Option<i32>,
    pub model: String,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub cost: Option<BigDecimal>,
    #[schemars(with = "ChatRsLogStatus")]
    pub status: String,
    pub meta: ChatRsLogMeta,
}

#[derive(Debug, Default, Clone, Copy, EnumString, AsRefStr, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ChatRsLogKind {
    Chat,
    Title,
    #[default]
    Prompt,
    Image,
}

#[derive(Debug, Default, Clone, Copy, EnumString, AsRefStr, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ChatRsLogStatus {
    Started,
    Cancelled,
    Error,
    #[default]
    Completed,
}

#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize, FromSqlRow, AsExpression, AsJsonb, JsonSchema)]
#[diesel(sql_type = diesel::sql_types::Jsonb)]
pub struct ChatRsLogMeta {
    /// Options passed to the LLM provider
    pub options: Option<ChatRsLogMetaOptions>,
    /// Any errors received from the LLM provider
    pub errors: Option<Vec<String>>,
    /// The request ID at the LLM provider
    pub request_id: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct ChatRsLogMetaOptions {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::llm_logs)]
pub struct NewChatRsLog<'a> {
    pub kind: &'a str,
    pub user_id: &'a Uuid,
    pub provider_id: i32,
    pub session_id: Option<&'a Uuid>,
    pub model: &'a str,
    pub status: &'a str,
    pub meta: Option<&'a ChatRsLogMeta>,
    pub started_at: UtcDateTime,
}

#[derive(Default, Identifiable, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = super::schema::llm_logs)]
pub struct UpdateChatRsLog {
    pub id: i32,
    pub message_id: Option<Uuid>,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub ttft_ms: Option<i32>,
    pub cost: Option<BigDecimal>,
    pub status: String,
    pub meta: ChatRsLogMeta,
    pub completed_at: Option<UtcDateTime>,
}
