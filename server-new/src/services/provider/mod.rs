use std::{str::FromStr, sync::Arc};

use uuid::Uuid;

use crate::{
    db::{
        DbService,
        models::{ChatRsProvider, ChatRsProviderType, ChatRsSecret, OpenAISubtype},
    },
    llm::{
        interface::LlmProvider,
        providers::{LoremProvider, OpenAIProvider, OpenAIProviderConfig},
    },
    services::{auth::encryption::Encryptor, provider::error::ProviderError},
};

mod error;

pub struct ProviderService<'r> {
    encryptor: &'r Encryptor,
    http_client: &'r reqwest::Client,
}

impl<'r> ProviderService<'r> {
    pub fn new(encryptor: &'r Encryptor, http_client: &'r reqwest::Client) -> Self {
        Self {
            encryptor,
            http_client,
        }
    }

    pub async fn get_provider(
        &self,
        db: &mut DbService,
        user_id: &Uuid,
        provider_id: i32,
    ) -> Result<(ChatRsProvider, ChatRsProviderType, Option<ChatRsSecret>), ProviderError> {
        let (provider, api_key_secret) = db
            .providers()
            .find_by_id(user_id, provider_id)
            .await?
            .ok_or(ProviderError::NotFound)?;
        let provider_type = ChatRsProviderType::from_str(&provider.provider_type)?;

        Ok((provider, provider_type, api_key_secret))
    }

    pub async fn build_llm_provider(
        &self,
        db: &mut DbService,
        user_id: &Uuid,
        provider_id: i32,
    ) -> Result<Arc<dyn LlmProvider>, ProviderError> {
        let (provider, provider_type, api_key_secret) =
            self.get_provider(db, user_id, provider_id).await?;
        let api_key = api_key_secret
            .map(|secret| {
                self.encryptor
                    .decrypt_string(&secret.ciphertext, &secret.nonce)
            })
            .transpose()?;

        let llm_provider: Arc<dyn LlmProvider> = match provider_type {
            ChatRsProviderType::Lorem => Arc::new(LoremProvider::new()),
            ChatRsProviderType::OpenAI => Arc::new(OpenAIProvider::new(
                self.http_client,
                OpenAIProviderConfig::new(
                    provider
                        .openai_subtype
                        .and_then(|s| OpenAISubtype::from_str(&s).ok())
                        .unwrap_or_default(),
                    api_key.ok_or(ProviderError::MissingApiKey)?,
                    provider.base_url,
                ),
            )),
            _ => todo!(),
            // ChatRsProviderType::Anthropic => Box::new(AnthropicProvider::new(
            //     http_client,
            //     redis,
            //     api_key.ok_or(ProviderError::MissingApiKey)?,
            // )),
            // ChatRsProviderType::Ollama => Box::new(OllamaProvider::new(
            //     http_client,
            //     base_url.unwrap_or("http://localhost:11434"),
            // )),
        };

        Ok(llm_provider)
    }
}
