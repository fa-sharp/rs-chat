use std::{str::FromStr, sync::Arc};

use uuid::Uuid;

use crate::{
    db::{
        DbService,
        models::{
            ChatRsProvider, ChatRsProviderType, ChatRsSecret, NewChatRsProvider, NewChatRsSecret,
            OpenAISubtype, UpdateChatRsProvider, UpdateChatRsSecret,
        },
    },
    llm::{
        interface::LlmProvider,
        providers::{LoremProvider, OpenAIProvider, OpenAIProviderConfig},
    },
    services::{
        auth::encryption::Encryptor,
        provider::{
            error::ProviderError,
            types::{ProviderCreateInput, ProviderUpdateInput},
        },
    },
};

mod error;
pub mod types;

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

    pub async fn create_provider(
        &self,
        db: &mut DbService,
        user_id: &Uuid,
        input: &ProviderCreateInput,
    ) -> Result<ChatRsProvider, ProviderError> {
        let mut api_key_id: Option<Uuid> = None;
        if let Some(plaintext_key) = input.api_key.as_deref() {
            let (ciphertext, nonce) = self.encryptor.encrypt_string(plaintext_key)?;
            let secret_id = db
                .secrets()
                .create(NewChatRsSecret {
                    user_id,
                    name: &format!("{} API Key", input.name),
                    ciphertext: &ciphertext,
                    nonce: &nonce,
                })
                .await?;
            api_key_id = Some(secret_id);
        }
        let provider = db
            .providers()
            .create(NewChatRsProvider {
                name: &input.name,
                user_id,
                provider_type: input.r#type.into(),
                openai_subtype: input.openai_type.map(|t| t.into()),
                base_url: input.base_url.as_deref(),
                default_model: &input.default_model,
                api_key_id,
            })
            .await?;

        Ok(provider)
    }

    pub async fn update_provider(
        &self,
        db: &mut DbService,
        user_id: &Uuid,
        provider_id: i32,
        input: &ProviderUpdateInput,
    ) -> Result<ChatRsProvider, ProviderError> {
        let (provider, _, secret) = self.get_provider(db, user_id, provider_id).await?;

        let mut secret_id: Option<Uuid> = None;
        if let Some(new_api_key) = input.api_key.as_deref() {
            let (ciphertext, nonce) = self.encryptor.encrypt_string(new_api_key)?;
            secret_id = match secret {
                Some(existing_secret) => {
                    let update_secret = UpdateChatRsSecret {
                        ciphertext: Some(&ciphertext),
                        nonce: Some(&nonce),
                        ..Default::default()
                    };
                    let secret_id = db
                        .secrets()
                        .update(user_id, &existing_secret.id, update_secret)
                        .await?;
                    Some(secret_id)
                }
                None => {
                    let new_secret = NewChatRsSecret {
                        user_id,
                        name: &format!("{} API Key", provider.name),
                        ciphertext: &ciphertext,
                        nonce: &nonce,
                    };
                    let secret_id = db.secrets().create(new_secret).await?;
                    Some(secret_id)
                }
            };
        }

        let update_provider = UpdateChatRsProvider {
            api_key_id: secret_id,
            name: input.name.as_deref(),
            base_url: input.base_url.as_deref(),
            default_model: input.default_model.as_deref(),
        };
        let updated = db
            .providers()
            .update(&user_id, provider_id, update_provider)
            .await?;

        Ok(updated)
    }

    pub async fn delete_provider(
        &self,
        db: &mut DbService,
        user_id: &Uuid,
        provider_id: i32,
    ) -> Result<ChatRsProvider, ProviderError> {
        let (_provider, _, api_key_secret) = self.get_provider(db, user_id, provider_id).await?;
        if let Some(secret) = api_key_secret {
            db.secrets().delete(&user_id, &secret.id).await?;
        }
        let deleted = db.providers().delete(&user_id, provider_id).await?;

        Ok(deleted)
    }
}
