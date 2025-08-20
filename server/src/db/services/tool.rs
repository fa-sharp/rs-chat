use diesel::prelude::*;
use diesel::result::Error;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    models::{
        ChatRsExternalApiTool, ChatRsSecret, ChatRsSystemTool, NewChatRsExternalApiTool,
        NewChatRsSystemTool,
    },
    schema::{external_api_tools, secrets, system_tools},
    DbConnection,
};

pub struct ToolDbService<'a> {
    pub db: &'a mut DbConnection,
}

impl<'a> ToolDbService<'a> {
    pub fn new(db: &'a mut DbConnection) -> Self {
        ToolDbService { db }
    }

    pub async fn find_by_user(
        &mut self,
        user_id: &Uuid,
    ) -> Result<(Vec<ChatRsSystemTool>, Vec<ChatRsExternalApiTool>), Error> {
        let system_tools = system_tools::table
            .filter(system_tools::user_id.eq(user_id))
            .select(ChatRsSystemTool::as_select())
            .load(self.db)
            .await?;
        let external_api_tools = external_api_tools::table
            .filter(external_api_tools::user_id.eq(user_id))
            .select(ChatRsExternalApiTool::as_select())
            .load(self.db)
            .await?;

        Ok((system_tools, external_api_tools))
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

    pub async fn create_system_tool(
        &mut self,
        tool: NewChatRsSystemTool<'_>,
    ) -> Result<ChatRsSystemTool, Error> {
        diesel::insert_into(system_tools::table)
            .values(tool)
            .returning(ChatRsSystemTool::as_select())
            .get_result(self.db)
            .await
    }

    pub async fn create_external_api_tool(
        &mut self,
        tool: NewChatRsExternalApiTool<'_>,
    ) -> Result<ChatRsExternalApiTool, Error> {
        diesel::insert_into(external_api_tools::table)
            .values(tool)
            .returning(ChatRsExternalApiTool::as_select())
            .get_result(self.db)
            .await
    }

    pub async fn delete_system_tool(
        &mut self,
        user_id: &Uuid,
        tool_id: &Uuid,
    ) -> Result<Uuid, Error> {
        diesel::delete(system_tools::table)
            .filter(system_tools::user_id.eq(user_id))
            .filter(system_tools::id.eq(tool_id))
            .returning(system_tools::id)
            .get_result(self.db)
            .await
    }

    pub async fn delete_external_api_tool(
        &mut self,
        user_id: &Uuid,
        tool_id: &Uuid,
    ) -> Result<Uuid, Error> {
        diesel::delete(external_api_tools::table)
            .filter(external_api_tools::user_id.eq(user_id))
            .filter(external_api_tools::id.eq(tool_id))
            .returning(external_api_tools::id)
            .get_result(self.db)
            .await
    }

    pub async fn delete_by_user(&mut self, user_id: &Uuid) -> Result<Vec<Uuid>, Error> {
        let deleted_system_tools = diesel::delete(system_tools::table)
            .filter(system_tools::user_id.eq(user_id))
            .returning(system_tools::id)
            .get_results(self.db)
            .await?;
        let deleted_external_api_tools = diesel::delete(external_api_tools::table)
            .filter(external_api_tools::user_id.eq(user_id))
            .returning(external_api_tools::id)
            .get_results(self.db)
            .await?;

        Ok(deleted_system_tools
            .into_iter()
            .chain(deleted_external_api_tools.into_iter())
            .collect())
    }
}
