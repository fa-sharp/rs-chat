use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    models::{ChatRsFile, NewChatRsFile},
    schema::files,
    DbConnection,
};

pub struct FileDbService<'a> {
    pub db: &'a mut DbConnection,
}

impl<'a> FileDbService<'a> {
    pub fn new(db: &'a mut DbConnection) -> Self {
        Self { db }
    }

    pub async fn create_session_file(
        &mut self,
        file: NewChatRsFile<'_>,
    ) -> QueryResult<ChatRsFile> {
        diesel::insert_into(files::table)
            .values(file)
            .returning(ChatRsFile::as_returning())
            .get_result(self.db)
            .await
    }

    pub async fn find_session_file(
        &mut self,
        user_id: &Uuid,
        session_id: &Uuid,
        file_id: &Uuid,
    ) -> QueryResult<ChatRsFile> {
        files::table
            .filter(files::user_id.eq(user_id))
            .filter(files::session_id.eq(session_id))
            .filter(files::id.eq(file_id))
            .select(ChatRsFile::as_select())
            .first(self.db)
            .await
    }

    pub async fn list_session_files(
        &mut self,
        user_id: &Uuid,
        session_id: &Uuid,
    ) -> QueryResult<Vec<ChatRsFile>> {
        files::table
            .filter(files::user_id.eq(user_id))
            .filter(files::session_id.eq(session_id))
            .select(ChatRsFile::as_select())
            .load(self.db)
            .await
    }

    pub async fn delete_session_file(
        &mut self,
        user_id: &Uuid,
        session_id: &Uuid,
        file_id: &Uuid,
    ) -> QueryResult<Uuid> {
        diesel::delete(files::table)
            .filter(files::user_id.eq(user_id))
            .filter(files::session_id.eq(session_id))
            .filter(files::id.eq(file_id))
            .returning(files::id)
            .get_result(self.db)
            .await
    }
}
