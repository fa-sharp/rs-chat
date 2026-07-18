use diesel::prelude::*;
use schemars::JsonSchema;
use serde::Serialize;
use strum::{AsRefStr, EnumString};
use uuid::Uuid;

use crate::db::{
    UtcDateTime,
    models::{ChatRsMessage, ChatRsUser},
};

#[derive(Identifiable, Associations, Queryable, Selectable, Serialize, JsonSchema)]
#[diesel(belongs_to(ChatRsUser, foreign_key = user_id))]
#[diesel(table_name = super::schema::files)]
pub struct ChatRsFile {
    pub id: Uuid,
    #[serde(skip)]
    pub user_id: Uuid,
    pub session_id: Option<Uuid>,
    pub path: String,
    #[schemars(with = "ChatRsFileType")]
    pub file_type: String,
    pub content_type: String,
    pub size: i32,
    pub created_at: UtcDateTime,
    #[serde(skip)]
    pub updated_at: UtcDateTime,
}

#[derive(Identifiable, Selectable, Queryable, Associations, Serialize, JsonSchema)]
#[diesel(belongs_to(ChatRsMessage, foreign_key = message_id))]
#[diesel(belongs_to(ChatRsFile, foreign_key = file_id))]
#[diesel(table_name = super::schema::message_attachments)]
#[diesel(primary_key(message_id, file_id))]
pub struct ChatRsMessageAttachment {
    pub message_id: Uuid,
    pub file_id: Uuid,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::files)]
pub struct NewChatRsFile<'r> {
    pub user_id: &'r Uuid,
    pub session_id: Option<&'r Uuid>,
    pub path: &'r str,
    pub file_type: &'r str,
    pub content_type: &'r str,
    pub size: i32,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::message_attachments)]
pub struct NewChatRsMessageAttachment<'r> {
    pub message_id: &'r Uuid,
    pub file_id: &'r Uuid,
}

/// File modality
#[derive(Debug, PartialEq, Eq, Hash, EnumString, AsRefStr, JsonSchema)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ChatRsFileType {
    Text,
    Image,
    Pdf,
}
