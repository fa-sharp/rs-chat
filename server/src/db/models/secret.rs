use chrono::{DateTime, Utc};
use diesel::{
    prelude::{AsChangeset, Associations, Identifiable, Insertable, Queryable},
    Selectable,
};
use schemars::JsonSchema;
use uuid::Uuid;

use crate::db::models::ChatRsUser;

#[derive(Identifiable, Queryable, Selectable, Associations)]
#[diesel(belongs_to(ChatRsUser, foreign_key = user_id))]
#[diesel(table_name = super::schema::secrets)]
pub struct ChatRsSecret {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub created_at: DateTime<Utc>,
}

#[derive(Identifiable, Queryable, Selectable, Associations, JsonSchema, serde::Serialize)]
#[diesel(belongs_to(ChatRsUser, foreign_key = user_id))]
#[diesel(table_name = super::schema::secrets)]
pub struct ChatRsSecretMeta {
    pub id: Uuid,
    #[serde(skip)]
    pub user_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::secrets)]
pub struct NewChatRsSecret<'r> {
    pub user_id: &'r Uuid,
    pub name: &'r str,
    pub ciphertext: &'r Vec<u8>,
    pub nonce: &'r Vec<u8>,
}

#[derive(Default, AsChangeset)]
#[diesel(table_name = super::schema::secrets)]
pub struct UpdateChatRsSecret<'r> {
    pub name: Option<&'r str>,
    pub ciphertext: Option<&'r Vec<u8>>,
    pub nonce: Option<&'r Vec<u8>>,
}
