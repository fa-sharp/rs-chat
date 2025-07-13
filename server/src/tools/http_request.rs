use crate::{db::models::ChatRsTool, tools::ChatRsToolError};
use serde_json::Value;

pub struct ToolExecutor {
    http_client: reqwest::Client,
}

impl ToolExecutor {
    pub fn new(http_client: reqwest::Client) -> Self {
        Self { http_client }
    }

    pub async fn execute_tool(
        &self,
        tool: &ChatRsTool,
        parameters: &Value,
    ) -> Result<String, ChatRsToolError> {
        // Build the HTTP request components
        let url = self.build_url(&tool.url, parameters, &tool.query)?;
        let headers = self.build_headers(parameters)?;
        let body = self.build_body(parameters, &tool.body)?;

        // Execute the HTTP request
        let response = self
            .execute_request(&tool.method, &url, headers, body)
            .await?;

        Ok(response)
    }

    fn build_url(
        &self,
        base_url: &str,
        parameters: &Value,
        query_mapping: &Option<Value>,
    ) -> Result<String, ChatRsToolError> {
        let mut url = self.substitute_placeholders(base_url, parameters);

        // Add query parameters if mapping is provided
        if let Some(query_map) = query_mapping {
            let query_params = self.build_query_params(query_map, parameters)?;
            if !query_params.is_empty() {
                let separator = if url.contains('?') { "&" } else { "?" };
                url.push_str(separator);
                url.push_str(&query_params);
            }
        }

        Ok(url)
    }

    fn build_headers(
        &self,
        parameters: &Value,
    ) -> Result<reqwest::header::HeaderMap, ChatRsToolError> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        // TODO: Add support for custom headers with parameter substitution
        // This would require extending the tool schema to include header templates

        Ok(headers)
    }

    fn build_body(
        &self,
        parameters: &Value,
        body_mapping: &Option<Value>,
    ) -> Result<Option<String>, ChatRsToolError> {
        if let Some(body_template) = body_mapping {
            match body_template {
                Value::String(template) => {
                    let substituted = self.substitute_placeholders(template, parameters);
                    Ok(Some(substituted))
                }
                Value::Object(_) => {
                    // If body_mapping is an object, use it as a template for parameter mapping
                    let mapped_body = self.map_object_template(body_template, parameters)?;
                    Ok(Some(serde_json::to_string(&mapped_body)?))
                }
                _ => Ok(Some(serde_json::to_string(body_template)?)),
            }
        } else {
            Ok(None)
        }
    }

    fn build_query_params(
        &self,
        query_mapping: &Value,
        parameters: &Value,
    ) -> Result<String, ChatRsToolError> {
        let mut query_parts = Vec::new();

        if let Value::Object(mapping) = query_mapping {
            for (key, value_template) in mapping {
                if let Value::String(template) = value_template {
                    let substituted = self.substitute_placeholders(template, parameters);
                    if !substituted.is_empty() {
                        query_parts.push(format!(
                            "{}={}",
                            urlencoding::encode(key),
                            urlencoding::encode(&substituted)
                        ));
                    }
                }
            }
        }

        Ok(query_parts.join("&"))
    }

    fn substitute_placeholders(&self, template: &str, parameters: &Value) -> String {
        let mut result = template.to_string();

        if let Value::Object(params) = parameters {
            for (key, value) in params {
                let placeholder = format!("{{{{{}}}}}", key);
                let replacement = match value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => serde_json::to_string(value).unwrap_or_default(),
                };
                result = result.replace(&placeholder, &replacement);
            }
        }

        result
    }

    fn map_object_template(
        &self,
        template: &Value,
        parameters: &Value,
    ) -> Result<Value, ChatRsToolError> {
        match template {
            Value::Object(obj) => {
                let mut result = serde_json::Map::new();
                for (key, value) in obj {
                    result.insert(key.clone(), self.map_object_template(value, parameters)?);
                }
                Ok(Value::Object(result))
            }
            Value::Array(arr) => {
                let mut result = Vec::new();
                for item in arr {
                    result.push(self.map_object_template(item, parameters)?);
                }
                Ok(Value::Array(result))
            }
            Value::String(s) => {
                let substituted = self.substitute_placeholders(s, parameters);
                Ok(Value::String(substituted))
            }
            _ => Ok(template.clone()),
        }
    }

    async fn execute_request(
        &self,
        method: &str,
        url: &str,
        headers: reqwest::header::HeaderMap,
        body: Option<String>,
    ) -> Result<String, ChatRsToolError> {
        let request_builder = match method.to_uppercase().as_str() {
            "GET" => self.http_client.get(url),
            "POST" => self.http_client.post(url),
            "PUT" => self.http_client.put(url),
            "DELETE" => self.http_client.delete(url),
            "PATCH" => self.http_client.patch(url),
            _ => {
                return Err(ChatRsToolError::ToolExecutionError(format!(
                    "Unsupported HTTP method: {}",
                    method
                )))
            }
        };

        let mut request = request_builder.headers(headers);

        if let Some(body_content) = body {
            request = request.body(body_content);
        }

        let response = request.send().await.map_err(|e| {
            ChatRsToolError::ToolExecutionError(format!("Tool execution failed: {}", e))
        })?;

        let status = response.status();
        let response_text = response.text().await.map_err(|e| {
            ChatRsToolError::ToolExecutionError(format!("Failed to read response: {}", e))
        })?;

        if status.is_success() {
            Ok(response_text)
        } else {
            Err(ChatRsToolError::ToolExecutionError(format!(
                "Tool execution failed with status {}: {}",
                status, response_text
            )))
        }
    }
}
