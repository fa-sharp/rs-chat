use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    db::{
        models::{
            ChatRsMessage, ChatRsSession, NewChatRsMessage, NewChatRsSession, UpdateChatRsSession,
        },
        schema::{chat_messages, chat_sessions},
        DbConnection,
    },
    utils::full_text_search::{full_text_query, SessionSearchResult},
};

pub struct ChatDbService<'a> {
    pub db: &'a mut DbConnection,
}

impl<'a> ChatDbService<'a> {
    pub fn new(db: &'a mut DbConnection) -> Self {
        ChatDbService { db }
    }

    pub async fn create_session(
        &mut self,
        session: NewChatRsSession<'_>,
    ) -> Result<String, diesel::result::Error> {
        let id: Uuid = diesel::insert_into(chat_sessions::table)
            .values(session)
            .returning(chat_sessions::id)
            .get_result(self.db)
            .await?;
        Ok(id.to_string())
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

    pub async fn find_message(
        &mut self,
        user_id: &Uuid,
        message_id: &Uuid,
    ) -> Result<ChatRsMessage, diesel::result::Error> {
        chat_messages::table
            .inner_join(chat_sessions::table.on(chat_sessions::id.eq(chat_messages::session_id)))
            .select(ChatRsMessage::as_select())
            .filter(chat_sessions::user_id.eq(user_id))
            .filter(chat_messages::id.eq(message_id))
            .get_result(self.db)
            .await
    }

    pub async fn delete_message(
        &mut self,
        session_id: &Uuid,
        message_id: &Uuid,
    ) -> Result<String, diesel::result::Error> {
        let id: Uuid = diesel::delete(chat_messages::table)
            .filter(chat_messages::session_id.eq(session_id))
            .filter(chat_messages::id.eq(message_id))
            .returning(chat_messages::id)
            .get_result(self.db)
            .await?;
        Ok(id.to_string())
    }

    pub async fn get_all_sessions(
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

    pub async fn get_session(
        &mut self,
        user_id: &Uuid,
        session_id: &Uuid,
    ) -> Result<ChatRsSession, diesel::result::Error> {
        let session = chat_sessions::table
            .filter(chat_sessions::user_id.eq(user_id))
            .filter(chat_sessions::id.eq(session_id))
            .select(ChatRsSession::as_select())
            .first(self.db)
            .await?;

        Ok(session)
    }

    pub async fn get_session_with_messages(
        &mut self,
        user_id: &Uuid,
        session_id: &Uuid,
    ) -> Result<(ChatRsSession, Vec<ChatRsMessage>), diesel::result::Error> {
        let session = chat_sessions::table
            .filter(chat_sessions::user_id.eq(user_id))
            .filter(chat_sessions::id.eq(session_id))
            .select(ChatRsSession::as_select())
            .first(self.db)
            .await?;
        let messages = ChatRsMessage::belonging_to(&session)
            .select(ChatRsMessage::as_select())
            .order_by(chat_messages::created_at.asc())
            .load(self.db)
            .await?;

        Ok((session, messages))
    }

    pub async fn search_sessions(
        &mut self,
        user_id: &Uuid,
        query: &str,
    ) -> Result<Vec<SessionSearchResult>, diesel::result::Error> {
        let sessions = full_text_query(self.db, user_id, query, 10).await?;

        Ok(sessions)
    }

    pub async fn update_session(
        &mut self,
        user_id: &Uuid,
        session_id: &Uuid,
        data: UpdateChatRsSession<'_>,
    ) -> Result<Uuid, diesel::result::Error> {
        let updated_id: Uuid = diesel::update(chat_sessions::table.find(session_id))
            .set(data)
            .filter(chat_sessions::user_id.eq(user_id))
            .returning(chat_sessions::id)
            .get_result(self.db)
            .await?;

        Ok(updated_id)
    }

    pub async fn delete_session(
        &mut self,
        user_id: &Uuid,
        session_id: &Uuid,
    ) -> Result<Uuid, diesel::result::Error> {
        let id: Uuid = diesel::delete(chat_sessions::table.find(session_id))
            .filter(chat_sessions::user_id.eq(user_id))
            .returning(chat_sessions::id)
            .get_result(self.db)
            .await?;

        Ok(id)
    }

    pub async fn delete_all_sessions(
        &mut self,
        user_id: &Uuid,
    ) -> Result<Vec<Uuid>, diesel::result::Error> {
        let ids: Vec<Uuid> = diesel::delete(chat_sessions::table)
            .filter(chat_sessions::user_id.eq(user_id))
            .returning(chat_sessions::id)
            .get_results(self.db)
            .await?;

        Ok(ids)
    }
}
