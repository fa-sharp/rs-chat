use diesel::prelude::*;
use serde::Serialize;
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::db::UtcDateTime;

#[skip_serializing_none]
#[derive(Identifiable, Queryable, Selectable, Serialize)]
#[diesel(table_name = super::schema::users)]
pub struct ChatRsUser {
    pub id: Uuid,
    pub name: String,
    pub avatar_url: Option<String>,
    pub github_id: Option<String>,
    pub google_id: Option<String>,
    pub discord_id: Option<String>,
    pub oidc_id: Option<String>,
    pub sso_username: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: UtcDateTime,
}

#[derive(Insertable, Default)]
#[diesel(table_name = super::schema::users)]
pub struct NewChatRsUser<'r> {
    pub github_id: Option<&'r str>,
    pub google_id: Option<&'r str>,
    pub discord_id: Option<&'r str>,
    pub oidc_id: Option<&'r str>,
    pub sso_username: Option<&'r str>,
    pub name: &'r str,
    pub avatar_url: Option<&'r str>,
}

#[derive(AsChangeset, Default)]
#[diesel(table_name = super::schema::users)]
pub struct UpdateChatRsUser<'r> {
    pub github_id: Option<&'r str>,
    pub google_id: Option<&'r str>,
    pub discord_id: Option<&'r str>,
    pub oidc_id: Option<&'r str>,
    pub sso_username: Option<&'r str>,
    pub name: Option<&'r str>,
    pub avatar_url: Option<&'r str>,
}
