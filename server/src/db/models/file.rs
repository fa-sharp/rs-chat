use chrono::{DateTime, Utc};
use diesel::prelude::*;
use schemars::JsonSchema;
use uuid::Uuid;

use crate::{db::models::ChatRsUser, provider::LlmError};

#[derive(Identifiable, Associations, Queryable, Selectable, JsonSchema, serde::Serialize)]
#[diesel(belongs_to(ChatRsUser, foreign_key = user_id))]
#[diesel(table_name = super::schema::files)]
pub struct ChatRsFile {
    pub id: Uuid,
    #[serde(skip)]
    pub user_id: Uuid,
    #[serde(skip)]
    pub session_id: Option<Uuid>,
    pub path: String,
    #[schemars(with = "ChatRsFileType")]
    pub file_type: String,
    pub content_type: String,
    pub size: i32,
    pub created_at: DateTime<Utc>,
    #[serde(skip)]
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

impl TryFrom<&str> for ChatRsFileType {
    type Error = LlmError;

    fn try_from(file_type: &str) -> Result<Self, Self::Error> {
        match file_type {
            "text" => Ok(ChatRsFileType::Text),
            "image" => Ok(ChatRsFileType::Image),
            "pdf" => Ok(ChatRsFileType::Pdf),
            _ => Err(LlmError::InvalidFileType(file_type.to_owned())),
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
