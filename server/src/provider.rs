//! LLM providers module

use uuid::Uuid;

mod core;
pub use core::*;
pub mod models;
pub mod providers;
mod utils;

use crate::{
    db::{
        models::{ChatRsFileType, ChatRsMessage, ChatRsMessageRole, ChatRsProviderType},
        services::FileDbService,
        DbConnection,
    },
    errors::ApiError,
    provider::{models::LlmModel, providers::*},
    storage::LocalStorage,
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

/// Convert database messages to the generic messages to send to the provider implementation
pub async fn build_llm_messages(
    messages: Vec<ChatRsMessage>,
    user_id: &Uuid,
    session_id: &Uuid,
    db: &mut DbConnection,
    storage: &LocalStorage,
) -> Result<Vec<LlmMessage>, ApiError> {
    let mut llm_messages = Vec::with_capacity(messages.len());

    for message in messages {
        match message.role {
            ChatRsMessageRole::User => {
                let mut files: Option<Vec<LlmFileInput>> = None;
                if let Some(file_ids) = message.meta.user.and_then(|u| u.files) {
                    let mut file_db_service = FileDbService::new(db);
                    for file_id in file_ids {
                        let file = file_db_service
                            .find_session_file(user_id, session_id, &file_id)
                            .await?;
                        let (file_type, content) =
                            file.read_to_string(Some(session_id), storage).await?;
                        files.get_or_insert_default().push(LlmFileInput {
                            name: file.path,
                            content_type: file.content_type,
                            file_type,
                            content,
                        });
                    }
                }
                llm_messages.push(LlmMessage::User(LlmUserMessage {
                    text: message.content,
                    files,
                }))
            }
            ChatRsMessageRole::Assistant => {
                llm_messages.push(LlmMessage::Assistant(LlmAssistantMessage {
                    text: message.content,
                    tool_calls: message.meta.assistant.and_then(|a| a.tool_calls),
                }))
            }
            ChatRsMessageRole::System => llm_messages.push(LlmMessage::System(message.content)),
            ChatRsMessageRole::Tool => {
                if let Some(tool_call) = message.meta.tool_call {
                    llm_messages.push(LlmMessage::Tool(LlmToolResult {
                        tool_call_id: tool_call.id,
                        tool_name: tool_call.tool_name,
                        content: message.content,
                    }))
                }
            }
        }
    }

    Ok(llm_messages)
}
