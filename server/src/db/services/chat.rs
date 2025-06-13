use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use rocket::futures::try_join;
use uuid::Uuid;

use crate::db::{
    models::{ChatMessage, ChatSession, NewChatMessage, NewChatSession},
    schema::{chat_messages, chat_sessions},
    DbConnection,
};

pub struct ChatDbService<'a> {
    pub db: &'a mut DbConnection,
}

impl ChatDbService<'_> {
    pub async fn create_session(
        &mut self,
        session: NewChatSession<'_>,
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
        message: NewChatMessage<'_>,
    ) -> Result<String, diesel::result::Error> {
        let id: Uuid = diesel::insert_into(chat_messages::table)
            .values(message)
            .returning(chat_messages::id)
            .get_result(self.db)
            .await?;
        Ok(id.to_string())
    }

    pub async fn get_session(
        &mut self,
        session_id: &Uuid,
    ) -> Result<(ChatSession, Vec<ChatMessage>), diesel::result::Error> {
        let session_query = chat_sessions::table
            .filter(chat_sessions::id.eq(session_id))
            .select(ChatSession::as_select())
            .get_result(self.db);
        let messages_query = chat_messages::table
            .filter(chat_messages::session_id.eq(session_id))
            .select(ChatMessage::as_select())
            .load(self.db);

        let (session, messages) = try_join!(session_query, messages_query)?;

        Ok((session, messages))
    }
}
