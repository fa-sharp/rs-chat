use bigdecimal::{BigDecimal, FromPrimitive};
use bon::bon;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    db::{
        DbConnection, UtcDateTime,
        models::{ChatRsLogKind, ChatRsLogMeta, ChatRsLogStatus, NewChatRsLog, UpdateChatRsLog},
        schema::llm_logs,
    },
    llm::types::{LlmChatOptions, LlmUsage},
};

pub struct LogRepository<'a> {
    db: &'a mut DbConnection,
}

#[bon]
impl<'a> LogRepository<'a> {
    pub fn new(db: &'a mut DbConnection) -> Self {
        LogRepository { db }
    }

    /// Create a new LLM request log entry
    #[builder(finish_fn = "build")]
    pub async fn create(
        &mut self,
        user_id: &Uuid,
        provider_id: i32,
        llm_options: &LlmChatOptions,
        kind: ChatRsLogKind,
        session_id: Option<&Uuid>,
    ) -> QueryResult<i32> {
        let new_log = NewChatRsLog {
            kind: kind.as_ref(),
            user_id,
            provider_id,
            session_id,
            model: &llm_options.model,
            status: ChatRsLogStatus::Started.as_ref(),
            meta: Some(&ChatRsLogMeta {
                temperature: llm_options.temperature,
                max_tokens: llm_options.max_tokens,
            }),
        };

        diesel::insert_into(llm_logs::table)
            .values(new_log)
            .returning(llm_logs::id)
            .get_result(self.db)
            .await
    }

    /// Complete a LLM request log entry
    #[builder(finish_fn = "build")]
    pub async fn complete(
        &mut self,
        id: i32,
        message_id: Option<&Uuid>,
        request_id: Option<&str>,
        usage: Option<&LlmUsage>,
        error: Option<&str>,
        status: ChatRsLogStatus,
        completed_at: Option<UtcDateTime>,
    ) -> QueryResult<i32> {
        let update_log = UpdateChatRsLog {
            message_id,
            request_id,
            input_tokens: usage.and_then(|u| u.input_tokens),
            output_tokens: usage.and_then(|u| u.output_tokens),
            cost: usage.and_then(|u| u.cost.and_then(BigDecimal::from_f32)),
            error,
            status: status.as_ref(),
            completed_at: completed_at.unwrap_or_else(chrono::Utc::now),
        };

        diesel::update(llm_logs::table)
            .filter(llm_logs::id.eq(id))
            .set(update_log)
            .returning(llm_logs::id)
            .get_result(self.db)
            .await
    }
}
