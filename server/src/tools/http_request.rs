use std::{collections::HashMap, str::FromStr};

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::tools::ChatRsToolError;

pub struct HttpRequestTool {
    http_client: reqwest::Client,
    config: HttpRequestToolData,
}

#[derive(Debug, JsonSchema, Serialize, Deserialize)]
pub struct HttpRequestToolData {
    pub url: String,
    pub method: String,
    pub query: Option<HashMap<String, String>>,
    pub body: Option<serde_json::Value>,
    pub headers: Option<HashMap<String, String>>,
}

type Parameters = HashMap<String, serde_json::Value>;

impl HttpRequestTool {
    pub fn new(http_client: &reqwest::Client, config: HttpRequestToolData) -> Self {
        Self {
            http_client: http_client.clone(),
            config,
        }
    }

    pub async fn execute_tool(&self, parameters: &Parameters) -> Result<String, ChatRsToolError> {
        // Build the HTTP request components
        let url = self.build_url(parameters)?;
        let headers = self.build_headers(parameters)?;
        let body = self.build_body(parameters, &self.config.body)?;

        // Execute the HTTP request
        let response = self
            .execute_request(&self.config.method, &url, headers, body)
            .await?;

        Ok(response)
    }

    fn build_url(&self, parameters: &Parameters) -> Result<String, ChatRsToolError> {
        let mut url = self.substitute_placeholders(&self.config.url, parameters);

        let query_params = self.build_query_params(parameters)?;
        if !query_params.is_empty() {
            let separator = if url.contains('?') { "&" } else { "?" };
            url.push_str(separator);
            url.push_str(&query_params);
        }

        Ok(url)
    }

    fn build_headers(&self, parameters: &Parameters) -> Result<HeaderMap, ChatRsToolError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        if let Some(header_mapping) = &self.config.headers {
            for (key, template) in header_mapping {
                let value = self.substitute_placeholders(template, parameters);
                if !value.is_empty() {
                    let header_name = HeaderName::from_str(key).map_err(|_| {
                        ChatRsToolError::FormattingError(format!("Invalid header name: {}", key))
                    })?;
                    let header_value = HeaderValue::from_str(&value).map_err(|_| {
                        ChatRsToolError::FormattingError(format!("Invalid header value: {}", value))
                    })?;
                    headers.insert(header_name, header_value);
                }
            }
        }

        Ok(headers)
    }

    fn build_body(
        &self,
        parameters: &Parameters,
        body_mapping: &Option<serde_json::Value>,
    ) -> Result<Option<String>, ChatRsToolError> {
        if let Some(body_template) = body_mapping {
            match body_template {
                Value::Object(_) | Value::Array(_) | Value::String(_) => {
                    let mapped_body = self.map_object_template(body_template, parameters)?;
                    Ok(Some(serde_json::to_string(&mapped_body)?))
                }
                _ => Ok(Some(serde_json::to_string(body_template)?)),
            }
        } else {
            Ok(None)
        }
    }

    fn build_query_params(&self, parameters: &Parameters) -> Result<String, ChatRsToolError> {
        let mut query_parts = Vec::new();

        if let Some(query_mapping) = &self.config.query {
            for (key, template) in query_mapping {
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

        Ok(query_parts.join("&"))
    }

    fn substitute_placeholders(&self, template: &str, parameters: &Parameters) -> String {
        let mut result = template.to_string();

        for (key, value) in parameters {
            let placeholder = format!("{{{{{}}}}}", key);
            let replacement = match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => serde_json::to_string(value).unwrap_or_default(),
            };
            result = result.replace(&placeholder, &replacement);
        }

        result
    }

    fn map_object_template(
        &self,
        template: &Value,
        parameters: &Parameters,
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
        println!("Executing request {} {}", method, url);
        println!("Headers: {:?}", headers);
        println!("Body: {:?}", body);

        let request_builder = match method.to_uppercase().as_str() {
            "GET" => self.http_client.get(url),
            "POST" => self.http_client.post(url),
            "PUT" => self.http_client.put(url),
            "DELETE" => self.http_client.delete(url),
            "PATCH" => self.http_client.patch(url),
            _ => {
                return Err(ChatRsToolError::FormattingError(format!(
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
            ChatRsToolError::ToolExecutionError(format!("HTTP request failed: {}", e))
        })?;

        let status = response.status();
        let response_text = response.text().await.map_err(|e| {
            ChatRsToolError::ToolExecutionError(format!("Failed to read HTTP response: {}", e))
        })?;

        if status.is_success() {
            Ok(response_text)
        } else {
            Err(ChatRsToolError::ToolExecutionError(format!(
                "HTTP request failed with status {}: {}",
                status, response_text
            )))
        }
    }
}
