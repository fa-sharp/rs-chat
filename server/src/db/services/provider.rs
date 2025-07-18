use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    models::{ChatRsProvider, ChatRsSecret},
    schema::{providers, secrets},
    DbConnection,
};

pub struct ProviderDbService<'a> {
    pub db: &'a mut DbConnection,
}

impl<'a> ProviderDbService<'a> {
    pub fn new(db: &'a mut DbConnection) -> Self {
        ProviderDbService { db }
    }

    pub async fn get_by_id(
        &mut self,
        user_id: &Uuid,
        provider_id: i32,
    ) -> Result<(ChatRsProvider, Option<ChatRsSecret>), diesel::result::Error> {
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
    }
}
