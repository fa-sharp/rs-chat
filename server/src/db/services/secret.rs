use diesel::prelude::*;
use diesel::result::Error;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    models::{ChatRsProviderKeyType, ChatRsSecret, ChatRsSecretMeta, NewChatRsSecret},
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

    pub async fn find_by_user_and_provider(
        &mut self,
        user_id: &Uuid,
        provider: &ChatRsProviderKeyType,
    ) -> Result<Option<ChatRsSecret>, Error> {
        let key = secrets::table
            .filter(secrets::user_id.eq(user_id))
            .filter(secrets::provider.eq(provider))
            .select(ChatRsSecret::as_select())
            .first(self.db)
            .await
            .optional()?;

        Ok(key)
    }

    pub async fn create(&mut self, api_key: NewChatRsSecret<'_>) -> Result<Uuid, Error> {
        let id: Uuid = diesel::insert_into(secrets::table)
            .values(api_key)
            .returning(secrets::id)
            .get_result(self.db)
            .await?;

        Ok(id)
    }

    pub async fn delete(&mut self, user_id: &Uuid, api_key_id: &Uuid) -> Result<Uuid, Error> {
        let id: Uuid = diesel::delete(secrets::table)
            .filter(secrets::id.eq(api_key_id))
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
