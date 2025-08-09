use chrono::{DateTime, Utc};
use diesel::{
    prelude::{AsChangeset, Identifiable, Insertable, Queryable},
    Selectable,
};
use schemars::JsonSchema;
use uuid::Uuid;

#[derive(Identifiable, Queryable, Selectable, JsonSchema, serde::Serialize)]
#[diesel(table_name = super::schema::users)]
pub struct ChatRsUser {
    pub id: Uuid,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discord_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oidc_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sso_username: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
