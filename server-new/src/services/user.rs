use uuid::Uuid;

use crate::{
    db::{DbPool, DbService, models::ChatRsUser},
    error::AppResult,
};

pub struct UserService<'a> {
    db: &'a DbPool,
}

impl<'a> UserService<'a> {
    pub fn new(db: &'a DbPool) -> Self {
        Self { db }
    }

    pub async fn get_user(&self, id: &Uuid) -> AppResult<Option<ChatRsUser>> {
        let mut db = DbService::from_pool(&self.db).await?;
        let user = db.users().find_by_id(id).await?;

        Ok(user)
    }
}
