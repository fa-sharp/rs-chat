use diesel::prelude::*;
use diesel::result::Error;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    models::{ChatRsFile, NewChatRsAttachment, NewChatRsFile},
    schema::{chat_messages, chat_messages_attachments, files},
    DbConnection,
};

pub struct StorageDbService<'a> {
    pub db: &'a mut DbConnection,
}

impl<'a> StorageDbService<'a> {
    pub fn new(db: &'a mut DbConnection) -> Self {
        StorageDbService { db }
    }

    /// Find all attached files in this chat session
    pub async fn find_by_session(
        &mut self,
        session_id: &Uuid,
    ) -> Result<Vec<(Uuid, ChatRsFile)>, Error> {
        let files = chat_messages_attachments::table
            .inner_join(chat_messages::table)
            .inner_join(files::table)
            .filter(chat_messages::session_id.eq(session_id))
            .select((chat_messages::id, ChatRsFile::as_select()))
            .load(self.db)
            .await?;

        Ok(files)
    }

    pub async fn create(&mut self, file: NewChatRsFile<'_>) -> Result<ChatRsFile, Error> {
        diesel::insert_into(files::table)
            .values(file)
            .returning(ChatRsFile::as_returning())
            .get_result(self.db)
            .await
    }

    pub async fn attach_files_to_message(
        &mut self,
        user_id: &Uuid,
        message_id: &Uuid,
        file_ids: &Vec<Uuid>,
    ) -> Result<Vec<ChatRsFile>, Error> {
        let files_to_attach = files::table
            .filter(files::id.eq_any(file_ids))
            .filter(files::user_id.eq(user_id))
            .select(ChatRsFile::as_select())
            .load(self.db)
            .await?;
        let attachments: Vec<NewChatRsAttachment> = files_to_attach
            .iter()
            .map(|file| NewChatRsAttachment {
                file_id: &file.id,
                message_id,
            })
            .collect();

        diesel::insert_into(chat_messages_attachments::table)
            .values(attachments)
            .execute(self.db)
            .await?;

        Ok(files_to_attach)
    }

    pub async fn delete(&mut self, user_id: &Uuid, file_id: &Uuid) -> Result<Uuid, Error> {
        let id: Uuid = diesel::delete(files::table.find(file_id))
            .filter(files::user_id.eq(user_id))
            .returning(files::id)
            .get_result(self.db)
            .await?;

        Ok(id)
    }
}
