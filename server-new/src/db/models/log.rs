use bigdecimal::BigDecimal;
use diesel::prelude::*;
use schemars::JsonSchema;
use serde::Serialize;
use strum::{AsRefStr, EnumString};
use uuid::Uuid;

use crate::db::{UtcDateTime, models::ChatRsUser};

#[derive(Identifiable, Associations, Queryable, Selectable, Serialize, JsonSchema)]
#[diesel(belongs_to(ChatRsUser, foreign_key = user_id))]
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
    pub started_at: UtcDateTime,
    pub completed_at: Option<UtcDateTime>,
}

#[derive(Debug, Clone, Copy, EnumString, AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum ChatRsLogKind {
    Chat,
    Title,
    Prompt,
    Image,
}

#[derive(Debug, Clone, Copy, EnumString, AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum ChatRsLogStatus {
    Started,
    Completed,
    Cancelled,
    Error,
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
