use diesel::prelude::*;
use diesel::result::Error;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    models::{
        ChatRsExternalApiTool, ChatRsSecret, ChatRsSystemTool, ChatRsTool, ChatRsToolPublic,
        NewChatRsTool,
    },
    schema::{external_api_tools, secrets, system_tools, tools},
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

    pub async fn find_by_user_public(
        &mut self,
        user_id: &Uuid,
    ) -> Result<Vec<ChatRsToolPublic>, Error> {
        tools::table
            .filter(tools::user_id.eq(user_id))
            .select(ChatRsToolPublic::as_select())
            .order_by(tools::name.asc())
            .load(self.db)
            .await
    }

    pub async fn find_system_tool_by_id(
        &mut self,
        user_id: &Uuid,
        tool_id: &Uuid,
    ) -> Result<Option<ChatRsSystemTool>, Error> {
        system_tools::table
            .filter(system_tools::user_id.eq(user_id))
            .filter(system_tools::id.eq(tool_id))
            .select(ChatRsSystemTool::as_select())
            .first(self.db)
            .await
            .optional()
    }

    pub async fn find_system_tools_by_user(
        &mut self,
        user_id: &Uuid,
    ) -> Result<Vec<ChatRsSystemTool>, Error> {
        system_tools::table
            .filter(system_tools::user_id.eq(user_id))
            .select(ChatRsSystemTool::as_select())
            .load(self.db)
            .await
    }

    pub async fn find_external_api_tool_by_id(
        &mut self,
        user_id: &Uuid,
        tool_id: &Uuid,
    ) -> Result<Option<(ChatRsExternalApiTool, Option<ChatRsSecret>)>, Error> {
        external_api_tools::table
            .left_outer_join(
                secrets::table.on(external_api_tools::secret_1.eq(secrets::id.nullable())),
            )
            .filter(external_api_tools::user_id.eq(user_id))
            .filter(external_api_tools::id.eq(tool_id))
            .select((
                ChatRsExternalApiTool::as_select(),
                Option::<ChatRsSecret>::as_select(),
            ))
            .first(self.db)
            .await
            .optional()
    }

    pub async fn find_external_api_tools_by_user(
        &mut self,
        user_id: &Uuid,
    ) -> Result<Vec<ChatRsExternalApiTool>, Error> {
        external_api_tools::table
            .left_outer_join(
                secrets::table.on(external_api_tools::secret_1.eq(secrets::id.nullable())),
            )
            .filter(external_api_tools::user_id.eq(user_id))
            .select(ChatRsExternalApiTool::as_select())
            .load(self.db)
            .await
    }

    pub async fn create(&mut self, tool: NewChatRsTool<'_>) -> Result<ChatRsToolPublic, Error> {
        diesel::insert_into(tools::table)
            .values(tool)
            .returning(ChatRsToolPublic::as_select())
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
