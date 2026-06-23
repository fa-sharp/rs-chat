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
        user_id: Option<&Uuid>,
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
        diesel::update(auth_sessions::table.find(session_id))
            .set(UpdateChatRsAuthSession {
                data: AuthSessionData(data.to_owned()),
                expires_at,
            })
            .returning(ChatRsAuthSession::as_returning())
            .get_result(self.db)
            .await
    }

    /// Find an active (not expired) session by ID
    pub async fn find_active_by_id(
        &mut self,
        session_id: &Uuid,
    ) -> QueryResult<Option<ChatRsAuthSession>> {
        auth_sessions::table
            .find(session_id)
            .filter(auth_sessions::expires_at.gt(diesel::dsl::now))
            .select(ChatRsAuthSession::as_select())
            .first(self.db)
            .await
            .optional()
    }

    /// Delete a session by ID. Won't return an error if it does not exist.
    pub async fn delete_by_id(&mut self, session_id: &Uuid) -> QueryResult<usize> {
        diesel::delete(auth_sessions::table.find(session_id))
            .returning(auth_sessions::id)
            .execute(self.db)
            .await
    }
}
