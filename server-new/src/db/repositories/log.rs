use std::time::Duration;

use bigdecimal::{BigDecimal, FromPrimitive};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    db::{
        DbConnection, UtcDateTime,
        models::{
            ChatRsLogKind, ChatRsLogMeta, ChatRsLogMetaOptions, ChatRsLogStatus, NewChatRsLog,
            UpdateChatRsLog,
        },
        schema::llm_logs,
    },
    llm::types::{LlmChatOptions, LlmUsage},
};

pub struct LogRepository<'a> {
    db: &'a mut DbConnection,
}
impl<'a> LogRepository<'a> {
    pub fn new(db: &'a mut DbConnection) -> Self {
        LogRepository { db }
    }

    /// Create a new LLM request log entry
    pub async fn create(
        &mut self,
        LlmLogCreate {
            user_id,
            provider_id,
            llm_options,
            kind,
            session_id,
        }: LlmLogCreate<'_>,
    ) -> QueryResult<UpdateChatRsLog> {
        let new_log = NewChatRsLog {
            kind: kind.as_ref(),
            user_id: &user_id,
            provider_id,
            session_id,
            model: llm_options
                .as_ref()
                .map(|o| o.model.as_str())
                .unwrap_or_default(),
            status: ChatRsLogStatus::Started.as_ref(),
            meta: Some(&ChatRsLogMeta {
                options: Some(ChatRsLogMetaOptions {
                    temperature: llm_options.and_then(|o| o.temperature),
                    max_tokens: llm_options.and_then(|o| o.max_tokens),
                }),
                ..Default::default()
            }),
            started_at: chrono::Utc::now(),
        };

        diesel::insert_into(llm_logs::table)
            .values(new_log)
            .returning(UpdateChatRsLog::as_returning())
            .get_result(self.db)
            .await
    }

    /// Complete a LLM request log entry
    pub async fn complete(
        &mut self,
        log: UpdateChatRsLog,
        LlmLogComplete {
            message_id,
            request_id,
            usage,
            errors,
            first_token_in,
            status,
            completed_at,
        }: LlmLogComplete<'_>,
    ) -> QueryResult<i32> {
        let updated_log = UpdateChatRsLog {
            id: log.id,
            message_id,
            input_tokens: usage.and_then(|u| u.input_tokens),
            output_tokens: usage.and_then(|u| u.output_tokens),
            cost: usage.and_then(|u| u.cost.and_then(BigDecimal::from_f32)),
            status: status.as_ref().to_owned(),
            completed_at: Some(completed_at.unwrap_or_else(chrono::Utc::now)),
            ttft_ms: first_token_in.and_then(|d| d.as_millis().try_into().ok()),
            meta: ChatRsLogMeta {
                errors,
                request_id: request_id.map(str::to_owned),
                ..log.meta
            },
        };

        diesel::update(&updated_log)
            .set(&updated_log)
            .returning(llm_logs::id)
            .get_result(self.db)
            .await
    }
}

#[derive(Debug, Default)]
pub struct LlmLogCreate<'a> {
    pub kind: ChatRsLogKind,
    pub user_id: Uuid,
    pub provider_id: i32,
    pub session_id: Option<&'a Uuid>,
    pub llm_options: Option<&'a LlmChatOptions>,
}

#[derive(Debug, Default)]
pub struct LlmLogComplete<'a> {
    pub status: ChatRsLogStatus,
    pub message_id: Option<Uuid>,
    pub request_id: Option<&'a str>,
    pub usage: Option<&'a LlmUsage>,
    pub errors: Option<Vec<String>>,
    pub first_token_in: Option<Duration>,
    pub completed_at: Option<UtcDateTime>,
}
