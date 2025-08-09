use diesel::prelude::*;
use diesel::result::Error;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    models::{ChatRsSecretMeta, NewChatRsSecret, UpdateChatRsSecret},
    schema::secrets,
    DbConnection,
};

pub struct SecretDbService<'a> {
    pub db: &'a mut DbConnection,
}

impl<'a> SecretDbService<'a> {
    pub fn new(db: &'a mut DbConnection) -> Self {
        SecretDbService { db }
    }

    pub async fn find_by_user_id(
        &mut self,
        user_id: &Uuid,
    ) -> Result<Vec<ChatRsSecretMeta>, Error> {
        let keys = secrets::table
            .filter(secrets::user_id.eq(user_id))
            .select(ChatRsSecretMeta::as_select())
            .load(self.db)
            .await?;

        Ok(keys)
    }

    pub async fn create(&mut self, secret: NewChatRsSecret<'_>) -> Result<Uuid, Error> {
        let id: Uuid = diesel::insert_into(secrets::table)
            .values(secret)
            .returning(secrets::id)
            .get_result(self.db)
            .await?;

        Ok(id)
    }

    pub async fn update(
        &mut self,
        user_id: &Uuid,
        secret_id: &Uuid,
        data: UpdateChatRsSecret<'_>,
    ) -> Result<Uuid, Error> {
        let id: Uuid = diesel::update(secrets::table)
            .filter(secrets::id.eq(secret_id))
            .filter(secrets::user_id.eq(user_id))
            .set(data)
            .returning(secrets::id)
            .get_result(self.db)
            .await?;

        Ok(id)
    }

    pub async fn delete(&mut self, user_id: &Uuid, secret_id: &Uuid) -> Result<Uuid, Error> {
        let id: Uuid = diesel::delete(secrets::table)
            .filter(secrets::id.eq(secret_id))
            .filter(secrets::user_id.eq(user_id))
            .returning(secrets::id)
            .get_result(self.db)
            .await?;

        Ok(id)
    }

    pub async fn delete_by_user(&mut self, user_id: &Uuid) -> Result<Vec<Uuid>, Error> {
        let ids: Vec<Uuid> = diesel::delete(secrets::table)
            .filter(secrets::user_id.eq(user_id))
            .returning(secrets::id)
            .get_results(self.db)
            .await?;

        Ok(ids)
    }
}
