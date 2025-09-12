//! LLM providers module

mod core;
pub use core::*;
pub mod models;
pub mod providers;
mod utils;

use std::collections::HashMap;

use uuid::Uuid;

use crate::{
    db::{
        models::{ChatRsMessage, ChatRsMessageRole, ChatRsProviderType},
        services::FileDbService,
        DbConnection,
    },
    errors::ApiError,
    provider::{models::LlmModel, providers::*},
    storage::LocalStorage,
    tools::ToolError,
};

pub const DEFAULT_MAX_TOKENS: u32 = 2000;
pub const DEFAULT_TEMPERATURE: f32 = 0.7;

/// Build the LLM API to make calls to the provider
pub fn build_llm_provider_api(
    provider_type: &ChatRsProviderType,
    base_url: Option<&str>,
    api_key: Option<&str>,
    http_client: &reqwest::Client,
    redis: &fred::clients::Client,
) -> Result<Box<dyn LlmApiProvider>, LlmError> {
    match provider_type {
        ChatRsProviderType::Openai => Ok(Box::new(OpenAIProvider::new(
            http_client,
            redis,
            api_key.ok_or(LlmError::MissingApiKey)?,
            base_url,
        ))),
        ChatRsProviderType::Anthropic => Ok(Box::new(AnthropicProvider::new(
            http_client,
            redis,
            api_key.ok_or(LlmError::MissingApiKey)?,
        ))),
        ChatRsProviderType::Ollama => Ok(Box::new(OllamaProvider::new(
            http_client,
            base_url.unwrap_or("http://localhost:11434"),
        ))),
        ChatRsProviderType::Lorem => Ok(Box::new(LoremProvider::new())),
    }
}

/// Extract any attached files, then convert the database messages to the generic format
/// for sending to LLM providers
pub async fn build_llm_messages(
    messages: Vec<ChatRsMessage>,
    user_id: &Uuid,
    session_id: &Uuid,
    db: &mut DbConnection,
    storage: &LocalStorage,
) -> Result<Vec<LlmMessage>, ApiError> {
    // Get content of any attached files in the messages
    let mut file_map: HashMap<Uuid, LlmFileInput> = HashMap::new();
    let file_ids: Vec<Uuid> = messages.iter().fold(Vec::new(), |mut acc, message| {
        if let Some(file_ids) = message.meta.user.as_ref().and_then(|u| u.files.as_ref()) {
            acc.extend(file_ids);
        }
        acc
    });
    for file_id in file_ids {
        let file = FileDbService::new(db)
            .find_session_file(user_id, session_id, &file_id)
            .await?;
        let (file_type, content) = file.read_to_string(Some(session_id), storage).await?;
        file_map.insert(
            file_id,
            LlmFileInput {
                name: file.path,
                content_type: file.content_type,
                file_type,
                content,
            },
        );
    }

    // Convert the messages
    let llm_messages = messages
        .into_iter()
        .map(|message| match message.role {
            ChatRsMessageRole::User => {
                let files = message.meta.user.and_then(|u| u.files).map(|file_ids| {
                    file_ids
                        .iter()
                        .filter_map(|id| file_map.remove(id))
                        .collect()
                });
                Ok(LlmMessage::User(LlmUserMessage {
                    text: message.content,
                    files,
                }))
            }
            ChatRsMessageRole::Assistant => Ok(LlmMessage::Assistant(LlmAssistantMessage {
                text: message.content,
                tool_calls: message.meta.assistant.and_then(|a| a.tool_calls),
            })),
            ChatRsMessageRole::System => Ok(LlmMessage::System(message.content)),
            ChatRsMessageRole::Tool => {
                if let Some(tool_call) = message.meta.tool_call {
                    Ok(LlmMessage::Tool(LlmToolResult {
                        tool_call_id: tool_call.id,
                        tool_name: tool_call.tool_name,
                        content: message.content,
                    }))
                } else {
                    Err(ToolError::ToolCallNotFound)
                }
            }
        })
        .collect::<Result<Vec<LlmMessage>, ToolError>>()?;

    Ok(llm_messages)
}
