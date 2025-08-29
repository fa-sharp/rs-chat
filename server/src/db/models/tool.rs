use std::collections::HashMap;

use chrono::{DateTime, Utc};
use diesel::{
    prelude::{Associations, Identifiable, Insertable, Queryable},
    Selectable,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    db::models::ChatRsUser,
    provider::LlmToolType,
    tools::{ChatRsExternalApiToolConfig, ChatRsSystemToolConfig, ToolResponseFormat},
};

#[derive(Debug, Identifiable, Queryable, Selectable, Associations, Serialize, JsonSchema)]
#[diesel(belongs_to(ChatRsUser, foreign_key = user_id))]
#[diesel(table_name = super::schema::system_tools)]
pub struct ChatRsSystemTool {
    pub id: Uuid,
    pub user_id: Uuid,
    pub data: ChatRsSystemToolConfig,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::system_tools)]
pub struct NewChatRsSystemTool<'r> {
    pub user_id: &'r Uuid,
    pub data: &'r ChatRsSystemToolConfig,
}

#[derive(Debug, Identifiable, Queryable, Selectable, Associations, Serialize, JsonSchema)]
#[diesel(belongs_to(ChatRsUser, foreign_key = user_id))]
#[diesel(table_name = super::schema::external_api_tools)]
pub struct ChatRsExternalApiTool {
    pub id: Uuid,
    pub user_id: Uuid,
    pub data: ChatRsExternalApiToolConfig,
    pub secret_1: Option<Uuid>,
    pub secret_2: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::external_api_tools)]
pub struct NewChatRsExternalApiTool<'r> {
    pub user_id: &'r Uuid,
    pub data: &'r ChatRsExternalApiToolConfig,
    pub secret_1: Option<&'r Uuid>,
}

/// A tool call requested by the provider
#[derive(Debug, Clone, JsonSchema, serde::Serialize, serde::Deserialize)]
pub struct ChatRsToolCall {
    /// ID of the tool call
    pub id: String,
    /// ID of the tool used
    pub tool_id: Uuid,
    /// Name of the tool used
    pub tool_name: String,
    /// Type of the tool used
    #[serde(default)]
    pub tool_type: LlmToolType,
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
    #[serde(default)]
    pub tool_name: String,
    /// Type of the tool used
    #[serde(default)]
    pub tool_type: LlmToolType,
    /// Format of the tool response
    #[serde(default)]
    pub response_format: ToolResponseFormat,
    /// Whether the tool call resulted in an error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
    /// Collected logs from the tool execution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<Vec<String>>,
    /// Collected errors from the tool execution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
}
