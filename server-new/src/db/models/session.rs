use std::collections::HashMap;

use diesel::{
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    prelude::*,
    serialize::ToSql,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::{UtcDateTime, models::ChatRsUser};

#[derive(Identifiable, Associations, Queryable, Selectable)]
#[diesel(belongs_to(ChatRsUser, foreign_key = user_id))]
#[diesel(table_name = super::schema::auth_sessions)]
pub struct ChatRsAuthSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub data: AuthSessionData,
    pub expires_at: UtcDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::auth_sessions)]
pub struct NewChatRsAuthSession<'r> {
    pub id: &'r Uuid,
    pub user_id: &'r Uuid,
    pub data: AuthSessionData,
    pub expires_at: UtcDateTime,
}

#[derive(AsChangeset)]
#[diesel(table_name = super::schema::auth_sessions)]
pub struct UpdateChatRsAuthSession {
    pub data: AuthSessionData,
    pub expires_at: UtcDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromSqlRow, AsExpression)]
#[diesel(sql_type = diesel::sql_types::Jsonb)]
pub struct AuthSessionData(pub HashMap<String, serde_json::Value>);

impl FromSql<diesel::sql_types::Jsonb, diesel::pg::Pg> for AuthSessionData {
    fn from_sql(bytes: diesel::pg::PgValue<'_>) -> diesel::deserialize::Result<Self> {
        let value =
            <serde_json::Value as FromSql<diesel::sql_types::Jsonb, diesel::pg::Pg>>::from_sql(
                bytes,
            )?;
        Ok(serde_json::from_value(value)?)
    }
}

impl ToSql<diesel::sql_types::Jsonb, diesel::pg::Pg> for AuthSessionData {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>,
    ) -> diesel::serialize::Result {
        let value = serde_json::to_value(self)?;
        <serde_json::Value as ToSql<diesel::sql_types::Jsonb, diesel::pg::Pg>>::to_sql(
            &value,
            &mut out.reborrow(),
        )
    }
}
