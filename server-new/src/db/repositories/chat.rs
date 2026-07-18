use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    DbConnection,
    models::{
        ChatRsLogKind, ChatRsLogLlmRequest, ChatRsMessage, ChatRsSession, NewChatRsMessage,
        NewChatRsSession, UpdateChatRsSession,
    },
    queries::{FullTextSearchResult, full_text_query},
    schema::{chat_messages, chat_sessions, llm_logs},
};

pub struct ChatRepository<'a> {
    pub db: &'a mut DbConnection,
}

impl<'a> ChatRepository<'a> {
    pub fn new(db: &'a mut DbConnection) -> Self {
        ChatRepository { db }
    }

    pub async fn create_session(
        &mut self,
        session: NewChatRsSession<'_>,
    ) -> Result<Uuid, diesel::result::Error> {
        let id = diesel::insert_into(chat_sessions::table)
            .values(session)
            .returning(chat_sessions::id)
            .get_result(self.db)
            .await?;
        Ok(id)
    }

    pub async fn save_message(
        &mut self,
        message: NewChatRsMessage<'_>,
    ) -> Result<ChatRsMessage, diesel::result::Error> {
        let message = diesel::insert_into(chat_messages::table)
            .values(message)
            .returning(ChatRsMessage::as_select())
            .get_result(self.db)
            .await?;
        Ok(message)
    }

    pub async fn save_messages(
        &mut self,
        messages: &[NewChatRsMessage<'_>],
    ) -> Result<Vec<ChatRsMessage>, diesel::result::Error> {
        let messages = diesel::insert_into(chat_messages::table)
            .values(messages)
            .returning(ChatRsMessage::as_select())
            .get_results(self.db)
            .await?;
        Ok(messages)
    }

    pub async fn find_message(
        &mut self,
        user_id: &Uuid,
        message_id: &Uuid,
    ) -> Result<Option<ChatRsMessage>, diesel::result::Error> {
        chat_messages::table
            .inner_join(chat_sessions::table)
            .select(ChatRsMessage::as_select())
            .filter(chat_sessions::user_id.eq(user_id))
            .filter(chat_messages::id.eq(message_id))
            .get_result(self.db)
            .await
            .optional()
    }

    pub async fn delete_message(
        &mut self,
        session_id: &Uuid,
        message_id: &Uuid,
    ) -> Result<Option<Uuid>, diesel::result::Error> {
        diesel::delete(chat_messages::table)
            .filter(chat_messages::session_id.eq(session_id))
            .filter(chat_messages::id.eq(message_id))
            .returning(chat_messages::id)
            .get_result(self.db)
            .await
            .optional()
    }

    pub async fn list_recent_sessions(
        &mut self,
        user_id: &Uuid,
    ) -> Result<Vec<ChatRsSession>, diesel::result::Error> {
        let sessions = chat_sessions::table
            .filter(chat_sessions::user_id.eq(user_id))
            .select(ChatRsSession::as_select())
            .order_by(chat_sessions::updated_at.desc())
            .limit(100)
            .load(self.db)
            .await?;

        Ok(sessions)
    }

    pub async fn find_session(
        &mut self,
        user_id: &Uuid,
        session_id: &Uuid,
    ) -> Result<Option<ChatRsSession>, diesel::result::Error> {
        let session = chat_sessions::table
            .filter(chat_sessions::user_id.eq(user_id))
            .filter(chat_sessions::id.eq(session_id))
            .select(ChatRsSession::as_select())
            .first(self.db)
            .await
            .optional()?;

        Ok(session)
    }

    pub async fn list_messages(&mut self, session_id: &Uuid) -> QueryResult<Vec<ChatRsMessage>> {
        let messages = chat_messages::table
            .filter(chat_messages::session_id.eq(session_id))
            .select(ChatRsMessage::as_select())
            .order_by(chat_messages::created_at.asc())
            .load(self.db)
            .await?;

        Ok(messages)
    }

    pub async fn list_messages_with_logs(
        &mut self,
        session_id: &Uuid,
    ) -> QueryResult<Vec<(ChatRsMessage, Option<ChatRsLogLlmRequest>)>> {
        let messages = chat_messages::table
            .left_join(
                llm_logs::table.on(llm_logs::message_id
                    .eq(chat_messages::id.nullable())
                    .and(llm_logs::kind.eq(ChatRsLogKind::Chat.as_ref()))),
            )
            .filter(chat_messages::session_id.eq(session_id))
            .select((
                ChatRsMessage::as_select(),
                Option::<ChatRsLogLlmRequest>::as_select(),
            ))
            .order_by(chat_messages::created_at.asc())
            .load(self.db)
            .await?;

        Ok(messages)
    }

    pub async fn search_sessions(
        &mut self,
        user_id: &Uuid,
        query: &str,
    ) -> Result<Vec<FullTextSearchResult>, diesel::result::Error> {
        full_text_query(self.db, user_id, query, 10).await
    }

    pub async fn update_session(
        &mut self,
        user_id: &Uuid,
        session_id: &Uuid,
        data: UpdateChatRsSession<'_>,
    ) -> Result<Option<Uuid>, diesel::result::Error> {
        diesel::update(chat_sessions::table.find(session_id))
            .set(data)
            .filter(chat_sessions::user_id.eq(user_id))
            .returning(chat_sessions::id)
            .get_result(self.db)
            .await
            .optional()
    }

    pub async fn delete_session(
        &mut self,
        user_id: &Uuid,
        session_id: &Uuid,
    ) -> Result<Option<Uuid>, diesel::result::Error> {
        diesel::delete(chat_sessions::table.find(session_id))
            .filter(chat_sessions::user_id.eq(user_id))
            .returning(chat_sessions::id)
            .get_result(self.db)
            .await
            .optional()
    }

    pub async fn delete_sessions_by_user(
        &mut self,
        user_id: &Uuid,
    ) -> Result<usize, diesel::result::Error> {
        diesel::delete(chat_sessions::table)
            .filter(chat_sessions::user_id.eq(user_id))
            .execute(self.db)
            .await
    }
}
