use std::collections::HashMap;

use chrono::{DateTime, Utc};
use diesel::{
    prelude::{Associations, Identifiable, Insertable, Queryable},
    Selectable,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{db::models::ChatRsUser, tools::ToolConfig};

#[derive(Identifiable, Queryable, Selectable, Associations, JsonSchema, Serialize, Deserialize)]
#[diesel(belongs_to(ChatRsUser, foreign_key = user_id))]
#[diesel(table_name = super::schema::tools)]
pub struct ChatRsTool {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: String,
    pub config: ToolConfig,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::tools)]
pub struct NewChatRsTool<'r> {
    pub user_id: &'r Uuid,
    pub name: &'r str,
    pub description: &'r str,
    pub config: &'r ToolConfig,
}

/// A tool call requested by the provider
#[derive(Debug, JsonSchema, serde::Serialize, serde::Deserialize)]
pub struct ChatRsToolCall {
    /// ID of the tool call
    pub id: String,
    /// ID of the tool used
    pub tool_id: Uuid,
    /// Name of the tool used
    pub tool_name: String,
    /// Input parameters passed to the tool
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Metadata for an executed tool call
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ChatRsExecutedToolCall {
    /// ID of the tool call
    pub id: String,
    /// ID of the tool used
    pub tool_id: Uuid,
    /// Name of the tool used
    pub tool_name: String,
    /// Input parameters passed to the tool
    pub parameters: HashMap<String, serde_json::Value>,
    /// Whether the tool call resulted in an error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}
