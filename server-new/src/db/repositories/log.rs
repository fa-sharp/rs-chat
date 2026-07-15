use bigdecimal::{BigDecimal, FromPrimitive};
use bon::bon;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    db::{
        DbConnection,
        models::{ChatRsLogKind, ChatRsLogStatus, NewChatRsLog, UpdateChatRsLog},
        schema::logs,
    },
    llm::types::LlmUsage,
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
        model: &str,
        kind: ChatRsLogKind,
        session_id: Option<&Uuid>,
    ) -> QueryResult<i32> {
        let new_log = NewChatRsLog {
            kind: kind.as_ref(),
            user_id,
            provider_id,
            session_id,
            model,
            status: ChatRsLogStatus::Started.as_ref(),
        };

        diesel::insert_into(logs::table)
            .values(new_log)
            .returning(logs::id)
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
    ) -> QueryResult<i32> {
        let update_log = UpdateChatRsLog {
            message_id,
            request_id,
            input_tokens: usage
                .and_then(|u| u.input_tokens)
                .and_then(|t| t.try_into().ok()),
            output_tokens: usage
                .and_then(|u| u.output_tokens)
                .and_then(|t| t.try_into().ok()),
            cost: usage.and_then(|u| u.cost.and_then(BigDecimal::from_f32)),
            error,
            status: status.as_ref(),
            completed_at: Utc::now(),
        };

        diesel::update(logs::table)
            .filter(logs::id.eq(id))
            .set(update_log)
            .returning(logs::id)
            .get_result(self.db)
            .await
    }
}
