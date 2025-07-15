mod core;
mod http_request;
mod web_search;

pub use core::ToolError;

use std::collections::HashMap;

use diesel_as_jsonb::AsJsonb;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    db::models::ChatRsTool,
    tools::{
        core::{validate_json_schema, Tool, ToolJsonSchema, ToolParameters, ToolResult},
        http_request::{HttpRequestConfig, HttpRequestTool},
        web_search::{WebSearchConfig, WebSearchTool},
    },
};

/// Tool configuration stored in the daabase
#[derive(Debug, JsonSchema, Serialize, Deserialize, AsJsonb)]
#[serde(tag = "type")]
pub enum ToolConfig {
    Http(HttpRequestConfig),
    WebSearch(WebSearchConfig),
}

impl ToolConfig {
    /// Validate the tool's configuration
    pub fn validate(&mut self) -> Result<(), ToolError> {
        match self {
            ToolConfig::Http(config) => config.validate(),
            ToolConfig::WebSearch(config) => config.validate(),
        }
    }
}

impl ChatRsTool {
    /// Validate input parameters and execute the tool. Returns the response
    /// and a boolean indicating whether there was an error.
    pub async fn execute(
        &self,
        parameters: &HashMap<String, serde_json::Value>,
        http_client: &reqwest::Client,
    ) -> (String, Option<bool>) {
        let tool = self.create_tool_executor(http_client);
        if let Err(e) = tool.validate_input(parameters) {
            return (e.to_string(), Some(true));
        }

        rocket::info!("Executing tool: {}", tool.name());
        match tool.execute(parameters).await {
            Ok(response) => (response, None),
            Err(e) => (e.to_string(), Some(true)),
        }
    }

    /// Get the JSON schema for this tool's input parameters
    pub fn get_input_schema(&self) -> serde_json::Value {
        match &self.config {
            ToolConfig::Http(config) => config.get_input_schema(),
            ToolConfig::WebSearch(config) => config.get_input_schema(),
        }
    }

    /// Create the tool executor
    fn create_tool_executor(&self, http_client: &reqwest::Client) -> Box<dyn Tool + '_> {
        match &self.config {
            ToolConfig::Http(config) => Box::new(HttpRequestTool::new(http_client, config)),
            ToolConfig::WebSearch(config) => Box::new(WebSearchTool::new(http_client, config)),
        }
    }
}
