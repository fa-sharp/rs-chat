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

#[derive(Identifiable, Associations, Queryable, Selectable)]
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
    pub request_id: Option<String>,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub cost: Option<BigDecimal>,
    pub status: String,
    pub error: Option<String>,
    pub meta: Option<ChatRsLogMeta>,
    pub started_at: UtcDateTime,
    pub completed_at: Option<UtcDateTime>,
}

#[derive(Debug, Clone, Copy, EnumString, AsRefStr, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ChatRsLogKind {
    Chat,
    Title,
    Prompt,
    Image,
}

#[derive(Debug, Clone, Copy, EnumString, AsRefStr, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ChatRsLogStatus {
    Started,
    Completed,
    Cancelled,
    Error,
}

#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize, FromSqlRow, AsExpression, AsJsonb)]
#[diesel(sql_type = diesel::sql_types::Jsonb)]
pub struct ChatRsLogMeta {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[skip_serializing_none]
#[derive(Identifiable, Queryable, Selectable, Serialize, JsonSchema)]
#[diesel(table_name = super::schema::llm_logs)]
pub struct ChatRsLogLlmRequest {
    #[serde(skip)]
    pub id: i32,
    pub provider_id: Option<i32>,
    pub model: String,
    pub request_id: Option<String>,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub cost: Option<BigDecimal>,
    #[schemars(with = "ChatRsLogStatus")]
    pub status: String,
    pub error: Option<String>,
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
}

#[derive(Default, AsChangeset)]
#[diesel(table_name = super::schema::llm_logs)]
pub struct UpdateChatRsLog<'a> {
    pub message_id: Option<&'a Uuid>,
    pub request_id: Option<&'a str>,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub cost: Option<BigDecimal>,
    pub status: &'a str,
    pub error: Option<&'a str>,
    pub completed_at: UtcDateTime,
}
