use chrono::{DateTime, Utc};
use diesel::prelude::*;
use schemars::JsonSchema;
use uuid::Uuid;

use crate::db::models::ChatRsUser;

#[derive(Identifiable, Associations, Queryable, Selectable, JsonSchema, serde::Serialize)]
#[diesel(belongs_to(ChatRsUser, foreign_key = user_id))]
#[diesel(table_name = super::schema::files)]
pub struct ChatRsFile {
    pub id: Uuid,
    #[serde(skip_serializing)]
    pub user_id: Uuid,
    pub session_id: Option<Uuid>,
    pub path: String,
    #[schemars(with = "ChatRsFileType")]
    pub file_type: String,
    pub content_type: String,
    pub size: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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

/// File modality
#[derive(Debug, PartialEq, Eq, Hash, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ChatRsFileType {
    Text,
    Image,
    Pdf,
}

impl TryFrom<&'static str> for ChatRsFileType {
    type Error = &'static str;

    fn try_from(file_type: &'static str) -> Result<Self, Self::Error> {
        match file_type {
            "text" => Ok(ChatRsFileType::Text),
            "image" => Ok(ChatRsFileType::Image),
            "pdf" => Ok(ChatRsFileType::Pdf),
            _ => Err("Invalid file type"),
        }
    }
}

impl From<ChatRsFileType> for &'static str {
    fn from(file_type: ChatRsFileType) -> Self {
        match file_type {
            ChatRsFileType::Text => "text",
            ChatRsFileType::Image => "image",
            ChatRsFileType::Pdf => "pdf",
        }
    }
}
