use diesel::prelude::*;
use diesel::result::Error;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    models::{
        ChatRsProviderKey, ChatRsProviderKeyMeta, ChatRsProviderKeyType, NewChatRsProviderKey,
    },
    schema::api_keys,
    DbConnection,
};

pub struct ProviderKeyDbService<'a> {
    pub db: &'a mut DbConnection,
}

impl<'a> ProviderKeyDbService<'a> {
    pub fn new(db: &'a mut DbConnection) -> Self {
        ProviderKeyDbService { db }
    }

    pub async fn find_by_user_id(
        &mut self,
        user_id: &Uuid,
    ) -> Result<Vec<ChatRsProviderKeyMeta>, Error> {
        let keys = api_keys::table
            .filter(api_keys::user_id.eq(user_id))
            .select(ChatRsProviderKeyMeta::as_select())
            .load(self.db)
            .await?;

        Ok(keys)
    }

    pub async fn find_by_user_and_provider(
        &mut self,
        user_id: &Uuid,
        provider: &ChatRsProviderKeyType,
    ) -> Result<Option<ChatRsProviderKey>, Error> {
        let key = api_keys::table
            .filter(api_keys::user_id.eq(user_id))
            .filter(api_keys::provider.eq(provider))
            .select(ChatRsProviderKey::as_select())
            .first(self.db)
            .await
            .optional()?;

        Ok(key)
    }

    pub async fn create(&mut self, api_key: NewChatRsProviderKey<'_>) -> Result<Uuid, Error> {
        let id: Uuid = diesel::insert_into(api_keys::table)
            .values(api_key)
            .returning(api_keys::id)
            .get_result(self.db)
            .await?;

        Ok(id)
    }

    pub async fn delete(&mut self, user_id: &Uuid, api_key_id: &Uuid) -> Result<Uuid, Error> {
        let id: Uuid = diesel::delete(api_keys::table)
            .filter(api_keys::id.eq(api_key_id))
            .filter(api_keys::user_id.eq(user_id))
            .returning(api_keys::id)
            .get_result(self.db)
            .await?;

        Ok(id)
    }

    pub async fn delete_by_user(&mut self, user_id: &Uuid) -> Result<Vec<Uuid>, Error> {
        let ids: Vec<Uuid> = diesel::delete(api_keys::table)
            .filter(api_keys::user_id.eq(user_id))
            .returning(api_keys::id)
            .get_results(self.db)
            .await?;

        Ok(ids)
    }
}
