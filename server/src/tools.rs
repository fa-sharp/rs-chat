mod http_request;
mod web_search;

use std::collections::HashMap;

use diesel_as_jsonb::AsJsonb;
use rocket::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    db::models::ChatRsTool,
    tools::{
        http_request::{HttpRequestTool, HttpRequestToolData},
        web_search::{web_search_schema, WebSearchTool, WebSearchToolData},
    },
};

/// Tool configuration
#[derive(Debug, JsonSchema, Serialize, Deserialize, AsJsonb)]
#[serde(tag = "type")]
pub enum ChatRsToolData {
    Http(HttpRequestToolData),
    WebSearch(WebSearchToolData),
}

/// JSON schema for tool input parameters
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct ChatRsToolJsonSchema {
    r#type: ChatRsToolJsonSchemaType,
    properties: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    required: Option<Vec<String>>,
    #[serde(rename = "additionalProperties")]
    additional_properties: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
enum ChatRsToolJsonSchemaType {
    #[serde(rename = "object")]
    Object,
}

/// Tool-related errors
#[derive(Debug, thiserror::Error)]
pub enum ChatRsToolError {
    #[error("Invalid tool name")]
    InvalidToolName,
    #[error("Invalid JSON schema: {0}")]
    InvalidJsonSchema(String),
    #[error("Input parameters don't match JSON schema: {0}")]
    InvalidParameters(String),
    #[error("Tool not found")]
    ToolNotFound,
    #[error("Tool call not found")]
    ToolCallNotFound,
    #[error("Formatting error: {0}")]
    FormattingError(String),
    #[error("Serialization error")]
    SerializationError(#[from] serde_json::Error),
    #[error("Tool execution error: {0}")]
    ToolExecutionError(String),
}

/// Trait that all tools must implement for validation and execution
#[async_trait]
trait ChatRsToolExecutor {
    /// Get the JSON schema for the tool's input parameters
    fn input_schema(&self) -> serde_json::Value;

    /// Execute the tool with the given parameters
    async fn execute_tool(
        &self,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<String, ChatRsToolError>;

    /// Validate the input parameters
    fn validate_input(
        &self,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ChatRsToolError> {
        jsonschema::validate(&self.input_schema(), &serde_json::to_value(parameters)?)
            .map_err(|err| ChatRsToolError::InvalidParameters(err.to_string()))
    }
}

// Helper functions attached to the tool structs
impl ChatRsTool {
    /// Get the JSON schema for this tool's input parameters
    pub fn get_input_schema(&self) -> serde_json::Value {
        match &self.data {
            ChatRsToolData::Http(config) => {
                serde_json::to_value(&config.input_schema).expect("Should be valid JSON Schema")
            }
            ChatRsToolData::WebSearch(_) => web_search_schema(),
        }
    }

    /// Validate input parameters and execute the tool. Returns the response
    /// as a String and a boolean indicating whether there was an error.
    pub async fn execute(
        &self,
        parameters: &HashMap<String, serde_json::Value>,
        http_client: &reqwest::Client,
    ) -> (String, Option<bool>) {
        let tool_executor: Box<dyn ChatRsToolExecutor + Send + Sync> = match &self.data {
            ChatRsToolData::Http(http_request_config) => {
                Box::new(HttpRequestTool::new(http_client, http_request_config))
            }
            ChatRsToolData::WebSearch(web_search_config) => {
                Box::new(WebSearchTool::new(http_client, web_search_config))
            }
        };
        if let Err(e) = tool_executor.validate_input(parameters) {
            (e.to_string(), Some(true))
        } else {
            tool_executor
                .execute_tool(parameters)
                .await
                .map_or_else(|e| (e.to_string(), Some(true)), |response| (response, None))
        }
    }
}

impl ChatRsToolData {
    /// Validate the tool's configuration
    pub fn validate(&mut self) -> Result<(), ChatRsToolError> {
        match self {
            ChatRsToolData::Http(config) => validate_json_schema(&mut config.input_schema),
            ChatRsToolData::WebSearch(_) => Ok(()),
        }
    }
}

/// Ensure JSON schema is valid (using Draft 2020-12).
/// Also sets `additionalProperties` to false as required by OpenAI.
fn validate_json_schema(input_schema: &mut ChatRsToolJsonSchema) -> Result<(), ChatRsToolError> {
    input_schema.additional_properties = Some(false);
    jsonschema::draft202012::meta::validate(&serde_json::to_value(input_schema)?)
        .map_err(|e| ChatRsToolError::InvalidJsonSchema(e.to_string()))?;
    Ok(())
}
