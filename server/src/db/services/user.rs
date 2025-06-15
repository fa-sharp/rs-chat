use diesel::prelude::*;
use diesel::result::Error;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    models::{ChatRsUser, NewChatRsUser},
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

    pub async fn create(&mut self, user: NewChatRsUser<'_>) -> Result<ChatRsUser, Error> {
        diesel::insert_into(users::table)
            .values(user)
            .returning(ChatRsUser::as_returning())
            .get_result(self.db)
            .await
    }
}
