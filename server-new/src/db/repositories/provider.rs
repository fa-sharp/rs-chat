use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    DbConnection,
    models::{ChatRsProvider, ChatRsSecret, NewChatRsProvider, UpdateChatRsProvider},
    schema::{providers, secrets},
};

pub struct ProviderRepository<'a> {
    pub db: &'a mut DbConnection,
}

impl<'a> ProviderRepository<'a> {
    pub fn new(db: &'a mut DbConnection) -> Self {
        ProviderRepository { db }
    }

    pub async fn find_by_id(
        &mut self,
        user_id: &Uuid,
        provider_id: i32,
    ) -> Result<Option<(ChatRsProvider, Option<ChatRsSecret>)>, diesel::result::Error> {
        providers::table
            .left_join(secrets::table)
            .filter(providers::user_id.eq(user_id))
            .filter(providers::id.eq(provider_id))
            .select((
                ChatRsProvider::as_select(),
                Option::<ChatRsSecret>::as_select(),
            ))
            .first(self.db)
            .await
            .optional()
    }

    pub async fn list_by_user_id(
        &mut self,
        user_id: &Uuid,
    ) -> Result<Vec<ChatRsProvider>, diesel::result::Error> {
        providers::table
            .filter(providers::user_id.eq(user_id))
            .select(ChatRsProvider::as_select())
            .load(self.db)
            .await
    }

    pub async fn create(
        &mut self,
        provider: NewChatRsProvider<'_>,
    ) -> Result<ChatRsProvider, diesel::result::Error> {
        diesel::insert_into(providers::table)
            .values(provider)
            .returning(ChatRsProvider::as_returning())
            .get_result(self.db)
            .await
    }

    pub async fn update(
        &mut self,
        user_id: &Uuid,
        provider_id: i32,
        data: UpdateChatRsProvider<'_>,
    ) -> Result<ChatRsProvider, diesel::result::Error> {
        diesel::update(providers::table)
            .filter(providers::user_id.eq(user_id))
            .filter(providers::id.eq(provider_id))
            .set(data)
            .returning(ChatRsProvider::as_returning())
            .get_result(self.db)
            .await
    }

    pub async fn delete(
        &mut self,
        user_id: &Uuid,
        provider_id: i32,
    ) -> Result<ChatRsProvider, diesel::result::Error> {
        diesel::delete(providers::table)
            .filter(providers::user_id.eq(user_id))
            .filter(providers::id.eq(provider_id))
            .returning(ChatRsProvider::as_returning())
            .get_result(self.db)
            .await
    }

    pub async fn delete_by_user(
        &mut self,
        user_id: &Uuid,
    ) -> Result<Vec<ChatRsProvider>, diesel::result::Error> {
        diesel::delete(providers::table)
            .filter(providers::user_id.eq(user_id))
            .returning(ChatRsProvider::as_returning())
            .get_results(self.db)
            .await
    }
}
