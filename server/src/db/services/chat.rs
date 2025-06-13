use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    models::{ChatRsMessage, ChatRsSession, NewChatRsMessage, NewChatRsSession},
    schema::{chat_messages, chat_sessions},
    DbConnection,
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
    ) -> Result<String, diesel::result::Error> {
        let id: Uuid = diesel::insert_into(chat_messages::table)
            .values(message)
            .returning(chat_messages::id)
            .get_result(self.db)
            .await?;
        Ok(id.to_string())
    }

    pub async fn get_all_sessions(&mut self) -> Result<Vec<ChatRsSession>, diesel::result::Error> {
        let sessions = chat_sessions::table
            .select(ChatRsSession::as_select())
            .order_by(chat_sessions::created_at.desc())
            .limit(100)
            .load(self.db)
            .await?;

        Ok(sessions)
    }

    pub async fn get_session(
        &mut self,
        session_id: &Uuid,
    ) -> Result<(ChatRsSession, Vec<ChatRsMessage>), diesel::result::Error> {
        let session = chat_sessions::table
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
}
