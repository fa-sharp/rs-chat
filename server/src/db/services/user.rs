use diesel::prelude::*;
use diesel::result::Error;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    models::{ChatRsUser, NewChatRsUser, UpdateChatRsUser},
    schema::users,
    DbConnection,
};

pub struct UserDbService<'a> {
    pub db: &'a mut DbConnection,
}

impl<'a> UserDbService<'a> {
    pub fn new(db: &'a mut DbConnection) -> Self {
        UserDbService { db }
    }

    pub async fn find_by_id(&mut self, id: &Uuid) -> Result<Option<ChatRsUser>, Error> {
        let user = users::table
            .filter(users::id.eq(id))
            .select(ChatRsUser::as_select())
            .first(self.db)
            .await
            .optional()?;

        Ok(user)
    }

    pub async fn find_by_github_id(&mut self, id: u64) -> Result<Option<ChatRsUser>, Error> {
        let user = users::table
            .filter(users::github_id.eq(id.to_string()))
            .select(ChatRsUser::as_select())
            .first(self.db)
            .await
            .optional()?;

        Ok(user)
    }

    pub async fn find_by_google_id(&mut self, id: &str) -> Result<Option<ChatRsUser>, Error> {
        let user = users::table
            .filter(users::google_id.eq(id))
            .select(ChatRsUser::as_select())
            .first(self.db)
            .await
            .optional()?;

        Ok(user)
    }

    pub async fn find_by_discord_id(&mut self, id: &str) -> Result<Option<ChatRsUser>, Error> {
        let user = users::table
            .filter(users::discord_id.eq(id))
            .select(ChatRsUser::as_select())
            .first(self.db)
            .await
            .optional()?;

        Ok(user)
    }

    pub async fn find_by_proxy_username(
        &mut self,
        username: &str,
    ) -> Result<Option<ChatRsUser>, Error> {
        let user = users::table
            .filter(users::proxy_username.eq(username))
            .select(ChatRsUser::as_select())
            .first(self.db)
            .await
            .optional()?;

        Ok(user)
    }

    pub async fn create(&mut self, user: NewChatRsUser<'_>) -> Result<ChatRsUser, Error> {
        diesel::insert_into(users::table)
            .values(user)
            .returning(ChatRsUser::as_returning())
            .get_result(self.db)
            .await
    }

    pub async fn update(
        &mut self,
        user_id: &Uuid,
        data: UpdateChatRsUser<'_>,
    ) -> Result<Uuid, diesel::result::Error> {
        let updated_id: Uuid = diesel::update(users::table.find(user_id))
            .set(data)
            .returning(users::id)
            .get_result(self.db)
            .await?;

        Ok(updated_id)
    }

    pub async fn delete(&mut self, user_id: &Uuid) -> Result<Uuid, Error> {
        let id: Uuid = diesel::delete(users::table.find(user_id))
            .returning(users::id)
            .get_result(self.db)
            .await?;

        Ok(id)
    }
}
