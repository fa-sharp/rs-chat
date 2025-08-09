use chrono::{DateTime, Utc};
use diesel::{
    prelude::{Associations, Identifiable, Insertable, Queryable},
    Selectable,
};
use schemars::JsonSchema;
use uuid::Uuid;

use crate::db::models::ChatRsUser;

#[derive(Identifiable, Queryable, Selectable, Associations, JsonSchema, serde::Serialize)]
#[diesel(belongs_to(ChatRsUser, foreign_key = user_id))]
#[diesel(table_name = super::schema::app_api_keys)]
pub struct ChatRsApiKey {
    pub id: Uuid,
    #[serde(skip)]
    pub user_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::app_api_keys)]
pub struct NewChatRsApiKey<'r> {
    pub user_id: &'r Uuid,
    pub name: &'r str,
}
