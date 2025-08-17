use std::{collections::HashMap, str::FromStr};

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use rocket::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use subst::VariableMap;

use crate::{
    provider::{LlmTool, LlmToolType},
    tools::{
        utils::{http_request_builder::HttpRequestBuilder, validate_json_schema},
        ToolJsonSchema,
    },
    utils::sender_with_logging::SenderWithLogging,
};

use super::{
    ExternalApiTool, ExternalApiToolConfig, ToolError, ToolLog, ToolParameters, ToolResult,
};

/// Custom API tool that is a collection of HTTP requests
pub struct CustomApiTool<'a> {
    config: &'a CustomApiConfig,
}

/// Saved configuration for the custom API tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CustomApiConfig {
    /// Name of the custom API tool. Will be prefixed to the request names.
    name: String,
    /// Map of request names to their configurations
    tools: HashMap<String, HttpRequestConfig>,
}

/// Dynamic configuration for the custom API tool
#[derive(Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CustomApiDynamicConfig {
    /// Which requests/tools are enabled
    enabled: Option<Vec<String>>,
}

/// Configuration for individual HTTP requests
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct HttpRequestConfig {
    description: String,
    url: String,
    method: String,
    query: Option<HashMap<String, String>>,
    body: Option<serde_json::Value>,
    headers: Option<HashMap<String, String>>,
    input_schema: ToolJsonSchema,
}

impl ExternalApiToolConfig for CustomApiConfig {
    type DynamicConfig = CustomApiDynamicConfig;

    fn get_llm_tools(
        &self,
        tool_id: uuid::Uuid,
        input_config: Option<&CustomApiDynamicConfig>,
    ) -> Vec<LlmTool> {
        self.tools
            .iter()
            .filter(|(name, _)| {
                input_config
                    .as_ref()
                    .and_then(|c| c.enabled.as_ref())
                    .map_or(true, |enabled| enabled.contains(name))
            })
            .map(|(name, config)| LlmTool {
                tool_id,
                tool_type: LlmToolType::ExternalApi,
                name: format!("{}:{}", self.name, name),
                description: config.description.clone(),
                input_schema: serde_json::to_value(&config.input_schema)
                    .expect("Should already be validated"),
            })
            .collect()
    }

    fn validate(&mut self) -> ToolResult<()> {
        for (_, config) in self.tools.iter_mut() {
            validate_json_schema(&mut config.input_schema)?;
        }
        Ok(())
    }
}

#[async_trait]
impl ExternalApiTool for CustomApiTool<'_> {
    fn input_schema(&self, tool_name: &str) -> ToolResult<serde_json::Value> {
        let http_request = self
            .config
            .tools
            .get(tool_name.split_once(':').ok_or(ToolError::ToolNotFound)?.1)
            .ok_or(ToolError::ToolNotFound)?;
        Ok(serde_json::to_value(&http_request.input_schema)?)
    }

    async fn execute(
        &self,
        tool_name: &str,
        parameters: &ToolParameters,
        _secrets: &[String],
        http_client: &reqwest::Client,
        tx: &SenderWithLogging<ToolLog>,
    ) -> Result<String, ToolError> {
        let request_config = self
            .config
            .tools
            .get(tool_name.split_once(':').ok_or(ToolError::ToolNotFound)?.1)
            .ok_or(ToolError::ToolNotFound)?;

        // Build the HTTP request components
        let _ = tx.send(ToolLog::Log("Building request...".into())).await;
        let url = request_config.build_url(parameters)?;
        let headers = request_config.build_headers(parameters)?;
        let body = request_config.build_body(parameters, &request_config.body)?;

        // Execute the HTTP request
        let _ = tx.send(ToolLog::Log("Sending request...".into())).await;
        match self
            .execute_request(http_client, &request_config.method, &url, headers, body)
            .await
        {
            Ok(response) => {
                let _ = tx.send(ToolLog::Log("Success!".into())).await;
                Ok(response)
            }
            Err(err) => {
                let _ = tx.send(ToolLog::Error(err.to_string())).await;
                Err(err)
            }
        }
    }
}

impl<'a> CustomApiTool<'a> {
    pub fn new(config: &'a CustomApiConfig) -> Self {
        Self { config }
    }

    async fn execute_request(
        &self,
        http_client: &reqwest::Client,
        method: &str,
        url: &str,
        headers: reqwest::header::HeaderMap,
        body: Option<String>,
    ) -> Result<String, ToolError> {
        let mut request = HttpRequestBuilder::new(method, url).headers(headers);
        if let Some(body_content) = body {
            request = request.body(body_content);
        }
        let response = request.send(http_client).await?;
        Ok(response)
    }
}

impl HttpRequestConfig {
    fn build_url(&self, parameters: &ToolParameters) -> Result<String, ToolError> {
        let param_map = ParameterMap(parameters);
        let url = subst::substitute(&self.url, &param_map)
            .map_err(|e| ToolError::FormattingError(format!("URL templating failed: {}", e)))?;

        let query_params = self.build_query_params(parameters)?;
        if !query_params.is_empty() {
            let separator = if url.contains('?') { "&" } else { "?" };
            Ok(format!("{}{}{}", url, separator, query_params))
        } else {
            Ok(url)
        }
    }

    fn build_headers(&self, parameters: &ToolParameters) -> Result<HeaderMap, ToolError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        if let Some(header_mapping) = &self.headers {
            let param_map = ParameterMap(parameters);

            for (key, template) in header_mapping {
                let value = subst::substitute(template, &param_map).map_err(|e| {
                    ToolError::FormattingError(format!("Header templating failed: {}", e))
                })?;

                if !value.is_empty() {
                    let header_name = HeaderName::from_str(key).map_err(|_| {
                        ToolError::FormattingError(format!("Invalid header name: {}", key))
                    })?;
                    let header_value = HeaderValue::from_str(&value).map_err(|_| {
                        ToolError::FormattingError(format!("Invalid header value: {}", value))
                    })?;
                    headers.insert(header_name, header_value);
                }
            }
        }

        Ok(headers)
    }

    fn build_body(
        &self,
        parameters: &ToolParameters,
        body_template: &Option<serde_json::Value>,
    ) -> Result<Option<String>, ToolError> {
        if let Some(template) = body_template {
            // First pass: direct value injection for exact parameter matches
            let mut body = self.apply_direct_injection(template, parameters)?;

            // Second pass: string substitution for partial matches
            let param_map = ParameterMap(parameters);
            subst::json::substitute_string_values(&mut body, &param_map).map_err(|e| {
                ToolError::FormattingError(format!("Body templating failed: {}", e))
            })?;

            Ok(Some(serde_json::to_string(&body)?))
        } else {
            Ok(None)
        }
    }

    fn build_query_params(&self, parameters: &ToolParameters) -> Result<String, ToolError> {
        let mut query_parts = Vec::new();

        if let Some(query_mapping) = &self.query {
            let param_map = ParameterMap(parameters);

            for (key, template) in query_mapping {
                let substituted = subst::substitute(template, &param_map).map_err(|e| {
                    ToolError::FormattingError(format!("Query templating failed: {}", e))
                })?;

                if !substituted.is_empty() {
                    query_parts.push(format!(
                        "{}={}",
                        urlencoding::encode(key),
                        urlencoding::encode(&substituted)
                    ));
                }
            }
        }

        Ok(query_parts.join("&"))
    }

    fn apply_direct_injection(
        &self,
        template: &serde_json::Value,
        parameters: &ToolParameters,
    ) -> Result<serde_json::Value, ToolError> {
        match template {
            serde_json::Value::Object(obj) => {
                let mut result = serde_json::Map::new();
                for (key, value) in obj {
                    result.insert(key.clone(), self.apply_direct_injection(value, parameters)?);
                }
                Ok(serde_json::Value::Object(result))
            }
            serde_json::Value::Array(arr) => {
                let mut result = Vec::new();
                for item in arr {
                    result.push(self.apply_direct_injection(item, parameters)?);
                }
                Ok(serde_json::Value::Array(result))
            }
            serde_json::Value::String(s) => {
                // Check if this is an exact parameter reference for direct injection
                if let Some(param_name) = s.strip_prefix('$').filter(|name| {
                    // Only do direct injection if it's the entire string (no other text)
                    name.chars().all(|c| c.is_alphanumeric() || c == '_')
                }) {
                    // Direct value injection - use the parameter value as-is
                    Ok(parameters
                        .get(param_name)
                        .cloned()
                        .unwrap_or(serde_json::Value::Null))
                } else {
                    // Keep as string for later substitution by subst
                    Ok(template.clone())
                }
            }
            _ => Ok(template.clone()),
        }
    }
}

/// Wrapper to make our parameters work with subst
struct ParameterMap<'a>(&'a ToolParameters);

impl<'a> VariableMap<'_> for ParameterMap<'a> {
    type Value = String;

    fn get(&self, key: &str) -> Option<Self::Value> {
        self.0.get(key).map(|value| match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            _ => serde_json::to_string(value).unwrap_or_default(),
        })
    }
}
// #[cfg(test)]
// mod tests {
//     use crate::tools::core::ToolJsonSchemaType;

//     use super::*;
//     use serde_json::json;
//     use std::collections::HashMap;

//     fn create_test_parameters() -> ToolParameters {
//         let mut params = HashMap::new();
//         params.insert("user_id".to_string(), json!("123"));
//         params.insert("name".to_string(), json!("John"));
//         params.insert("tags".to_string(), json!(["admin", "user"]));
//         params.insert("count".to_string(), json!(42));
//         params.insert("enabled".to_string(), json!(true));
//         params
//     }

//     #[test]
//     fn test_direct_injection() {
//         let config = HttpRequestConfig {
//             input_schema: ToolJsonSchema {
//                 r#type: ToolJsonSchemaType::Object,
//                 properties: HashMap::new(),
//                 required: None,
//                 additional_properties: Some(false),
//             },
//             url: "https://api.example.com".to_string(),
//             method: "POST".to_string(),
//             query: None,
//             body: Some(json!({
//                 "user": "$user_id",
//                 "tags": "$tags",
//                 "count": "$count",
//                 "enabled": "$enabled"
//             })),
//             headers: None,
//         };

//         let client = reqwest::Client::new();
//         let tool = CustomApiTool::new(&client, &config);
//         let params = create_test_parameters();

//         let result = tool
//             .apply_direct_injection(&config.body.as_ref().unwrap(), &params)
//             .unwrap();

//         assert_eq!(result["user"], json!("123"));
//         assert_eq!(result["tags"], json!(["admin", "user"]));
//         assert_eq!(result["count"], json!(42));
//         assert_eq!(result["enabled"], json!(true));
//     }

//     #[test]
//     fn test_string_templating() {
//         let config = HttpRequestConfig {
//             input_schema: ToolJsonSchema {
//                 r#type: crate::tools::core::ToolJsonSchemaType::Object,
//                 properties: HashMap::new(),
//                 required: None,
//                 additional_properties: Some(false),
//             },
//             url: "https://api.example.com/users/${user_id}".to_string(),
//             method: "GET".to_string(),
//             query: None,
//             body: Some(json!({
//                 "message": "Hello ${name}!",
//                 "info": "User ${user_id} has ${count} items"
//             })),
//             headers: Some({
//                 let mut headers = HashMap::new();
//                 headers.insert("Authorization".to_string(), "Bearer ${token}".to_string());
//                 headers
//             }),
//         };

//         let mut params = create_test_parameters();
//         params.insert("token".to_string(), json!("abc123"));

//         let client = reqwest::Client::new();
//         let tool = CustomApiTool::new(&client, &config);

//         // Test URL templating
//         let url = tool.build_url(&params).unwrap();
//         assert_eq!(url, "https://api.example.com/users/123");

//         // Test headers templating
//         let headers = tool.build_headers(&params).unwrap();
//         assert_eq!(headers["Authorization"], "Bearer abc123");

//         // Test body templating (after direct injection)
//         let mut body = tool
//             .apply_direct_injection(&config.body.as_ref().unwrap(), &params)
//             .unwrap();
//         let param_map = ParameterMap(&params);
//         subst::json::substitute_string_values(&mut body, &param_map).unwrap();

//         assert_eq!(body["message"], json!("Hello John!"));
//         assert_eq!(body["info"], json!("User 123 has 42 items"));
//     }

//     #[test]
//     fn test_mixed_templating() {
//         let body_template = json!({
//             "user": "$user_id",           // Direct injection
//             "greeting": "Hello ${name}!", // String templating
//             "tags": "$tags",              // Direct injection (array)
//             "summary": "${name} has ${count} items" // String templating
//         });

//         let config = HttpRequestConfig {
//             input_schema: ToolJsonSchema {
//                 r#type: crate::tools::core::ToolJsonSchemaType::Object,
//                 properties: HashMap::new(),
//                 required: None,
//                 additional_properties: Some(false),
//             },
//             url: "https://api.example.com".to_string(),
//             method: "POST".to_string(),
//             query: None,
//             body: Some(body_template),
//             headers: None,
//         };

//         let client = reqwest::Client::new();
//         let tool = CustomApiTool::new(&client, &config);
//         let params = create_test_parameters();

//         let body_result = tool.build_body(&params, &config.body).unwrap().unwrap();
//         let parsed: serde_json::Value = serde_json::from_str(&body_result).unwrap();

//         println!("Parsed body: {:?}", parsed);
//         assert_eq!(parsed["user"], json!("123")); // Direct injection preserved string
//         assert_eq!(parsed["greeting"], json!("Hello John!")); // String templating
//         assert_eq!(parsed["tags"], json!(["admin", "user"])); // Direct injection preserved array
//         assert_eq!(parsed["summary"], json!("John has 42 items")); // String templating
//     }
// }
