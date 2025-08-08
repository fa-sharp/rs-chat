use diesel::prelude::*;
use diesel::result::Error;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    models::{ChatRsTool, NewChatRsTool},
    schema::tools,
    DbConnection,
};

pub struct ToolDbService<'a> {
    pub db: &'a mut DbConnection,
}

impl<'a> ToolDbService<'a> {
    pub fn new(db: &'a mut DbConnection) -> Self {
        ToolDbService { db }
    }

    pub async fn find_by_id(
        &mut self,
        user_id: &Uuid,
        tool_id: &Uuid,
    ) -> Result<Option<ChatRsTool>, Error> {
        tools::table
            .filter(tools::user_id.eq(user_id))
            .filter(tools::id.eq(tool_id))
            .select(ChatRsTool::as_select())
            .first(self.db)
            .await
            .optional()
    }

    pub async fn find_by_user(&mut self, user_id: &Uuid) -> Result<Vec<ChatRsTool>, Error> {
        tools::table
            .filter(tools::user_id.eq(user_id))
            .select(ChatRsTool::as_select())
            .order_by(tools::name.asc())
            .load(self.db)
            .await
    }

    pub async fn create(&mut self, tool: NewChatRsTool<'_>) -> Result<ChatRsTool, Error> {
        diesel::insert_into(tools::table)
            .values(tool)
            .returning(ChatRsTool::as_select())
            .get_result(self.db)
            .await
    }

    pub async fn delete(&mut self, user_id: &Uuid, tool_id: &Uuid) -> Result<Uuid, Error> {
        diesel::delete(tools::table)
            .filter(tools::user_id.eq(user_id))
            .filter(tools::id.eq(tool_id))
            .returning(tools::id)
            .get_result(self.db)
            .await
    }

    pub async fn delete_by_user(&mut self, user_id: &Uuid) -> Result<Vec<Uuid>, Error> {
        diesel::delete(tools::table)
            .filter(tools::user_id.eq(user_id))
            .returning(tools::id)
            .get_results(self.db)
            .await
    }
}
