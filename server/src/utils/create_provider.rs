#![allow(deprecated)]

use llm::builder::LLMBackend;
use schemars::JsonSchema;
use uuid::Uuid;

use crate::{
    db::{models::ChatRsProviderKeyType, services::ProviderKeyDbService},
    errors::ApiError,
    provider::{
        anthropic::AnthropicProvider,
        llm::LlmApiProvider,
        lorem::{LoremConfig, LoremProvider},
        openai::OpenAIProvider,
        ChatRsError, ChatRsProvider,
    },
    utils::encryption::Encryptor,
};

const OPENROUTER_API_BASE_URL: &str = "https://openrouter.ai/api/v1";

/// Provider configuration input from API
// WARNING: This enum is also used to store metadata for chat messages in the database.
// Changes should be made carefully.
#[derive(Debug, Clone, JsonSchema, serde::Serialize, serde::Deserialize)]
pub enum ProviderConfigInput {
    Lorem,
    Anthropic(AnthropicConfig),
    OpenAI(OpenAIConfig),
    OpenRouter(OpenRouterConfig),
    #[deprecated]
    Llm(LLMConfig),
}

#[derive(Debug, Clone, JsonSchema, serde::Serialize, serde::Deserialize)]
pub struct AnthropicConfig {
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, JsonSchema, serde::Serialize, serde::Deserialize)]
pub struct OpenAIConfig {
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, JsonSchema, serde::Serialize, serde::Deserialize)]
pub struct OpenRouterConfig {
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[deprecated]
#[derive(Debug, Clone, JsonSchema, serde::Serialize, serde::Deserialize)]
pub struct LLMConfig {
    pub backend: LLMBackendInput,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

pub async fn create_provider<'a>(
    user_id: &Uuid,
    provider_config: &'a ProviderConfigInput,
    db: &mut ProviderKeyDbService<'_>,
    encryptor: &Encryptor,
    http_client: &reqwest::Client,
) -> Result<Box<dyn ChatRsProvider + Send + 'a>, ApiError> {
    let provider: Box<dyn ChatRsProvider + Send> = match provider_config {
        ProviderConfigInput::Lorem => Box::new(LoremProvider {
            config: LoremConfig { interval: 400 },
        }),
        ProviderConfigInput::Anthropic(anthropic_config) => {
            let api_key =
                find_key(user_id, ChatRsProviderKeyType::Anthropic, db, encryptor).await?;
            Box::new(AnthropicProvider::new(
                http_client,
                &api_key,
                &anthropic_config.model,
                anthropic_config.max_tokens,
                anthropic_config.temperature,
            )?)
        }
        ProviderConfigInput::OpenAI(openai_config) => {
            let api_key = find_key(user_id, ChatRsProviderKeyType::Openai, db, encryptor).await?;
            Box::new(OpenAIProvider::new(
                http_client,
                &api_key,
                &openai_config.model,
                openai_config.max_tokens,
                openai_config.temperature,
                openai_config.base_url.as_deref(),
            )?)
        }
        ProviderConfigInput::OpenRouter(config) => {
            let api_key =
                find_key(user_id, ChatRsProviderKeyType::Openrouter, db, encryptor).await?;
            Box::new(OpenAIProvider::new(
                http_client,
                &api_key,
                &config.model,
                config.max_tokens,
                config.temperature,
                Some(OPENROUTER_API_BASE_URL),
            )?)
        }
        ProviderConfigInput::Llm(llm_config) => {
            let api_key =
                find_key(user_id, llm_config.backend.clone().into(), db, encryptor).await?;

            Box::new(LlmApiProvider::new(
                llm_config.backend.clone().into(),
                api_key.to_owned(),
                &llm_config.model,
                llm_config.max_tokens,
                llm_config.temperature,
            ))
        }
    };

    Ok(provider)
}

async fn find_key(
    user_id: &Uuid,
    key_type: ChatRsProviderKeyType,
    db: &mut ProviderKeyDbService<'_>,
    encryptor: &Encryptor,
) -> Result<String, ApiError> {
    let api_key = db
        .find_by_user_and_provider(user_id, &key_type)
        .await?
        .ok_or(ChatRsError::MissingApiKey)
        .map(|key| encryptor.decrypt_string(&key.ciphertext, &key.nonce))??;
    Ok(api_key)
}

#[derive(Clone, Debug, JsonSchema, serde::Serialize, serde::Deserialize)]
pub enum LLMBackendInput {
    OpenAI,
    Anthropic,
    Deepseek,
    Google,
}

impl From<LLMBackendInput> for LLMBackend {
    fn from(value: LLMBackendInput) -> Self {
        match value {
            LLMBackendInput::OpenAI => LLMBackend::OpenAI,
            LLMBackendInput::Anthropic => LLMBackend::Anthropic,
            LLMBackendInput::Deepseek => LLMBackend::DeepSeek,
            LLMBackendInput::Google => LLMBackend::Google,
        }
    }
}

impl From<LLMBackendInput> for ChatRsProviderKeyType {
    fn from(value: LLMBackendInput) -> Self {
        match value {
            LLMBackendInput::OpenAI => ChatRsProviderKeyType::Openai,
            LLMBackendInput::Anthropic => ChatRsProviderKeyType::Anthropic,
            LLMBackendInput::Deepseek => ChatRsProviderKeyType::Deepseek,
            LLMBackendInput::Google => ChatRsProviderKeyType::Google,
        }
    }
}
