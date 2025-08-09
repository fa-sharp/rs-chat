use diesel::prelude::*;
use diesel::result::Error;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    models::{ChatRsApiKey, NewChatRsApiKey},
    schema::app_api_keys,
    DbConnection,
};

pub struct ApiKeyDbService<'a> {
    pub db: &'a mut DbConnection,
}

impl<'a> ApiKeyDbService<'a> {
    pub fn new(db: &'a mut DbConnection) -> Self {
        ApiKeyDbService { db }
    }

    pub async fn find_by_id(&mut self, id: &Uuid) -> Result<Option<ChatRsApiKey>, Error> {
        app_api_keys::table
            .find(id)
            .select(ChatRsApiKey::as_select())
            .first(self.db)
            .await
            .optional()
    }

    pub async fn find_by_user_id(&mut self, user_id: &Uuid) -> Result<Vec<ChatRsApiKey>, Error> {
        let keys = app_api_keys::table
            .filter(app_api_keys::user_id.eq(user_id))
            .select(ChatRsApiKey::as_select())
            .load(self.db)
            .await?;

        Ok(keys)
    }

    pub async fn create(&mut self, api_key: NewChatRsApiKey<'_>) -> Result<Uuid, Error> {
        let id: Uuid = diesel::insert_into(app_api_keys::table)
            .values(api_key)
            .returning(app_api_keys::id)
            .get_result(self.db)
            .await?;

        Ok(id)
    }

    pub async fn delete(&mut self, user_id: &Uuid, api_key_id: &Uuid) -> Result<Uuid, Error> {
        let id: Uuid = diesel::delete(app_api_keys::table)
            .filter(app_api_keys::id.eq(api_key_id))
            .filter(app_api_keys::user_id.eq(user_id))
            .returning(app_api_keys::id)
            .get_result(self.db)
            .await?;

        Ok(id)
    }

    pub async fn delete_by_user(&mut self, user_id: &Uuid) -> Result<Vec<Uuid>, Error> {
        let ids: Vec<Uuid> = diesel::delete(app_api_keys::table)
            .filter(app_api_keys::user_id.eq(user_id))
            .returning(app_api_keys::id)
            .get_results(self.db)
            .await?;

        Ok(ids)
    }
}
