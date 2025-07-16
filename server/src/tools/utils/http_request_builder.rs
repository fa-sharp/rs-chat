use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::str::FromStr;

use crate::tools::{core::ToolResult, ToolError};

/// Generic HTTP request builder that can be reused across tools
pub struct HttpRequestBuilder {
    method: String,
    url: String,
    headers: HeaderMap,
    body: Option<String>,
}

impl HttpRequestBuilder {
    pub fn new(method: &str, url: &str) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        Self {
            method: method.to_uppercase(),
            url: url.to_string(),
            headers,
            body: None,
        }
    }

    pub fn header(mut self, key: &str, value: &str) -> ToolResult<Self> {
        let header_name = HeaderName::from_str(key)
            .map_err(|_| ToolError::FormattingError(format!("Invalid header name: {}", key)))?;
        let header_value = HeaderValue::from_str(value)
            .map_err(|_| ToolError::FormattingError(format!("Invalid header value: {}", value)))?;
        self.headers.insert(header_name, header_value);
        Ok(self)
    }

    pub fn body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }

    pub fn query_param(mut self, key: &str, value: &str) -> Self {
        let separator = if self.url.contains('?') { "&" } else { "?" };
        self.url.push_str(&format!(
            "{}{}={}",
            separator,
            urlencoding::encode(key),
            urlencoding::encode(value)
        ));
        self
    }

    pub async fn send(self, client: &reqwest::Client) -> ToolResult<String> {
        let request_builder = match self.method.as_str() {
            "GET" => client.get(&self.url),
            "POST" => client.post(&self.url),
            "PUT" => client.put(&self.url),
            "DELETE" => client.delete(&self.url),
            "PATCH" => client.patch(&self.url),
            _ => {
                return Err(ToolError::FormattingError(format!(
                    "Unsupported HTTP method: {}",
                    self.method
                )))
            }
        };

        let mut request = request_builder.headers(self.headers);
        if let Some(body) = self.body {
            request = request.body(body);
        }

        let response = request
            .send()
            .await
            .map_err(|e| ToolError::ToolExecutionError(format!("HTTP request failed: {}", e)))?;

        let status = response.status();
        let response_text = response.text().await.map_err(|e| {
            ToolError::ToolExecutionError(format!("Failed to read response: {}", e))
        })?;

        if status.is_success() {
            Ok(response_text)
        } else {
            Err(ToolError::ToolExecutionError(format!(
                "Request failed with status {}: {}",
                status, response_text
            )))
        }
    }
}
