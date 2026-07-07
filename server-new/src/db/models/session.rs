use std::collections::HashMap;

use diesel::{deserialize::FromSqlRow, expression::AsExpression, prelude::*};
use diesel_jsonb_derive::AsJsonb;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::{UtcDateTime, models::ChatRsUser};

#[derive(Identifiable, Associations, Queryable, Selectable, Serialize, JsonSchema)]
#[diesel(belongs_to(ChatRsUser, foreign_key = user_id))]
#[diesel(table_name = super::schema::auth_sessions)]
pub struct ChatRsAuthSession {
    pub id: Uuid,
    #[serde(skip)]
    pub user_id: Option<Uuid>,
    pub data: AuthSessionData,
    pub expires_at: UtcDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::auth_sessions)]
pub struct NewChatRsAuthSession<'r> {
    pub id: &'r Uuid,
    pub user_id: Option<&'r Uuid>,
    pub data: AuthSessionData,
    pub expires_at: UtcDateTime,
}

#[derive(AsChangeset)]
#[diesel(table_name = super::schema::auth_sessions)]
pub struct UpdateChatRsAuthSession {
    pub data: AuthSessionData,
    pub expires_at: UtcDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromSqlRow, AsExpression, AsJsonb, JsonSchema)]
#[diesel(sql_type = diesel::sql_types::Jsonb)]
pub struct AuthSessionData(pub HashMap<String, serde_json::Value>);
