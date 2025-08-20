use rocket::async_trait;
use serde::Deserialize;

use crate::tools::utils::HttpRequestBuilder;

use super::{ToolError, ToolResult, WebSearchProvider, WebSearchResult};

pub struct ExaSearchTool {
    count: u8,
    max_characters: u32,
}
impl ExaSearchTool {
    pub fn new(count: u8, max_characters: u32) -> Self {
        Self {
            count,
            max_characters,
        }
    }
}
#[async_trait]
impl WebSearchProvider for ExaSearchTool {
    async fn search(
        &self,
        query: &str,
        api_key: &str,
        http_client: &reqwest::Client,
    ) -> Result<Vec<WebSearchResult>, ToolError> {
        let builder = HttpRequestBuilder::new("POST", "https://api.exa.ai/search")
            .header("X-Api-Key", api_key)?
            .body(
                serde_json::json!({
                    "query": query,
                    "type": "auto",
                    "numResults": self.count,
                    "contents": {
                        "text": {
                            "maxCharacters": 300
                        }
                    }
                })
                .to_string(),
            );

        let response_text = builder.send(http_client).await?;
        let exa_response: ExaSearchResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                ToolError::ToolExecutionError(format!("Failed to parse Exa response: {}", e))
            })?;

        Ok(exa_response
            .results
            .into_iter()
            .map(|result| WebSearchResult {
                title: result.title,
                url: result.url,
                text: result.text.unwrap_or_default(),
            })
            .collect())
    }

    async fn extract(
        &self,
        url: &str,
        api_key: &str,
        http_client: &reqwest::Client,
    ) -> ToolResult<String> {
        let builder = HttpRequestBuilder::new("POST", "https://api.exa.ai/contents")
            .header("X-Api-Key", api_key)?
            .body(
                serde_json::json!({
                    "ids": [url],
                    "text": { "maxCharacters": self.max_characters },
                    "extras": {
                        "links": 5,
                        "imageLinks": 0
                    }
                })
                .to_string(),
            );

        let response_text = builder.send(http_client).await?;
        let exa_response: ExaExtractResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                ToolError::ToolExecutionError(format!("Failed to parse Exa response: {}", e))
            })?;
        let exa_result = exa_response.results.first().ok_or_else(|| {
            ToolError::ToolExecutionError("No result found in Exa response".to_string())
        })?;

        let mut extracted_text = String::with_capacity(self.max_characters as usize);
        extracted_text.push_str(&format!(
            "Title: {}\nURL: {}\n\nContent:\n\n",
            &exa_result.title, &exa_result.url
        ));
        extracted_text.push_str(&exa_result.text);
        if let Some(links) = exa_result.extras.as_ref().and_then(|e| e.links.as_ref()) {
            extracted_text.push_str(&format!("\n\nLinks:\n"));
            for link in links {
                extracted_text.push_str(&format!("- {}\n", link));
            }
        }

        Ok(extracted_text)
    }
}

#[derive(Debug, Deserialize)]
struct ExaSearchResponse {
    results: Vec<ExaSearchResult>,
}

#[derive(Debug, Deserialize)]
struct ExaSearchResult {
    title: String,
    url: String,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ExaExtractResponse {
    results: Vec<ExaExtractResult>,
}

#[derive(Debug, Deserialize)]
struct ExaExtractResult {
    title: String,
    url: String,
    text: String,
    extras: Option<ExaExtractExtras>,
}

#[derive(Debug, Deserialize)]
struct ExaExtractExtras {
    links: Option<Vec<String>>,
}
