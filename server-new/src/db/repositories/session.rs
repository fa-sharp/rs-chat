use std::collections::HashMap;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    DbConnection, UtcDateTime,
    models::{AuthSessionData, ChatRsAuthSession, NewChatRsAuthSession, UpdateChatRsAuthSession},
    schema::auth_sessions,
};

pub struct SessionRepository<'a> {
    db: &'a mut DbConnection,
}

impl<'a> SessionRepository<'a> {
    pub fn new(db: &'a mut DbConnection) -> Self {
        Self { db }
    }

    pub async fn create(
        &mut self,
        session_id: &Uuid,
        user_id: &Uuid,
        data: &HashMap<String, serde_json::Value>,
        expires_at: UtcDateTime,
    ) -> QueryResult<ChatRsAuthSession> {
        diesel::insert_into(auth_sessions::table)
            .values(NewChatRsAuthSession {
                id: session_id,
                user_id,
                data: AuthSessionData(data.to_owned()),
                expires_at,
            })
            .returning(ChatRsAuthSession::as_returning())
            .get_result(self.db)
            .await
    }

    pub async fn update(
        &mut self,
        session_id: &Uuid,
        data: &HashMap<String, serde_json::Value>,
        expires_at: UtcDateTime,
    ) -> QueryResult<ChatRsAuthSession> {
        diesel::update(auth_sessions::table)
            .filter(auth_sessions::id.eq(session_id))
            .set(UpdateChatRsAuthSession {
                data: AuthSessionData(data.to_owned()),
                expires_at,
            })
            .returning(ChatRsAuthSession::as_returning())
            .get_result(self.db)
            .await
    }

    pub async fn find_by_id(
        &mut self,
        session_id: &Uuid,
    ) -> QueryResult<Option<ChatRsAuthSession>> {
        auth_sessions::table
            .find(session_id)
            .select(ChatRsAuthSession::as_select())
            .first(self.db)
            .await
            .optional()
    }

    pub async fn delete_by_id(&mut self, session_id: &Uuid) -> QueryResult<Uuid> {
        diesel::delete(auth_sessions::table.find(session_id))
            .returning(auth_sessions::id)
            .get_result(self.db)
            .await
    }
}
