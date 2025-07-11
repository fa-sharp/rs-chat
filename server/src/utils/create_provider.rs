use llm::builder::LLMBackend;
use schemars::JsonSchema;
use uuid::Uuid;

use crate::{
    db::{models::ChatRsApiKeyProviderType, services::api_key::ApiKeyDbService},
    provider::{
        llm::LlmApiProvider,
        lorem::{LoremConfig, LoremProvider},
        openrouter::OpenRouterProvider,
        ChatRsError, ChatRsProvider,
    },
    utils::encryption::Encryptor,
};

/// Provider configuration input from API
// WARNING: This enum is also used to store metadata for chat messages in the database.
// Changes should be made carefully.
#[derive(Debug, Clone, JsonSchema, serde::Serialize, serde::Deserialize)]
pub enum ProviderConfigInput {
    Lorem,
    Llm(LLMConfig),
    OpenRouter(OpenRouterConfig),
}

#[derive(Debug, Clone, JsonSchema, serde::Serialize, serde::Deserialize)]
pub struct LLMConfig {
    pub backend: LLMBackendInput,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}
impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            backend: LLMBackendInput::Anthropic,
            model: None,
            max_tokens: None,
            temperature: None,
        }
    }
}

#[derive(Debug, Clone, JsonSchema, serde::Serialize, serde::Deserialize)]
pub struct OpenRouterConfig {
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}
impl Default for OpenRouterConfig {
    fn default() -> Self {
        Self {
            model: None,
            max_tokens: None,
            temperature: None,
        }
    }
}

pub async fn create_provider<'a>(
    user_id: &Uuid,
    provider_config: &'a ProviderConfigInput,
    db: &mut ApiKeyDbService<'_>,
    encryptor: &Encryptor,
) -> Result<Box<dyn ChatRsProvider + Send + Sync + 'a>, ChatRsError> {
    let provider: Box<dyn ChatRsProvider + Send + Sync> = match provider_config {
        ProviderConfigInput::Lorem => Box::new(LoremProvider {
            config: LoremConfig { interval: 400 },
        }),
        ProviderConfigInput::Llm(llm_config) => {
            let api_key_secret = db
                .find_by_user_and_provider(user_id, &llm_config.backend.clone().into())
                .await
                .map_err(|e| ChatRsError::DatabaseError(e.to_string()))?
                .ok_or(ChatRsError::MissingApiKey)?;
            let api_key =
                encryptor.decrypt_string(&api_key_secret.ciphertext, &api_key_secret.nonce)?;

            Box::new(LlmApiProvider::new(
                llm_config.backend.clone().into(),
                api_key.to_owned(),
                llm_config.model.as_deref(),
                llm_config.max_tokens,
                llm_config.temperature,
            ))
        }
        ProviderConfigInput::OpenRouter(or_config) => {
            let api_key_secret = db
                .find_by_user_and_provider(user_id, &ChatRsApiKeyProviderType::Openrouter)
                .await
                .map_err(|e| ChatRsError::DatabaseError(e.to_string()))?
                .ok_or(ChatRsError::MissingApiKey)?;
            let api_key =
                encryptor.decrypt_string(&api_key_secret.ciphertext, &api_key_secret.nonce)?;

            Box::new(OpenRouterProvider::new(
                api_key,
                or_config.model.as_deref(),
                or_config.max_tokens,
                or_config.temperature,
            ))
        }
    };

    Ok(provider)
}

#[derive(Clone, Debug, JsonSchema, serde::Serialize, serde::Deserialize)]
pub enum LLMBackendInput {
    OpenAI,
    Anthropic,
    // Deepseek,
    // Google,
}

impl From<LLMBackendInput> for LLMBackend {
    fn from(value: LLMBackendInput) -> Self {
        match value {
            LLMBackendInput::OpenAI => LLMBackend::OpenAI,
            LLMBackendInput::Anthropic => LLMBackend::Anthropic,
            // LLMBackendInput::Deepseek => LLMBackend::DeepSeek,
            // LLMBackendInput::Google => LLMBackend::Google,
        }
    }
}

impl From<LLMBackendInput> for ChatRsApiKeyProviderType {
    fn from(value: LLMBackendInput) -> Self {
        match value {
            LLMBackendInput::OpenAI => ChatRsApiKeyProviderType::Openai,
            LLMBackendInput::Anthropic => ChatRsApiKeyProviderType::Anthropic,
            // LLMBackendInput::Deepseek => ChatRsApiKeyProviderType::Deepseek,
            // LLMBackendInput::Google => ChatRsApiKeyProviderType::Google,
        }
    }
}
