use std::{collections::HashMap, str::FromStr};

use fred::prelude::{HashesInterface, KeysInterface};
use serde::Deserialize;
use strum::{AsRefStr, EnumIter, IntoEnumIterator, IntoStaticStr};

use crate::{
    db::models::{ChatRsProvider, ChatRsProviderType, OpenAISubtype},
    services::model::{error::ModelError, types::LlmModel},
};

pub mod error;
mod providers;
pub mod types;

const MODELS_DEV_URL: &str = "https://models.dev/api.json";
const CACHE_KEY: &str = "rs-chat:models";
const CACHE_TTL: i64 = 86400; // 1 day in seconds

/// Service for fetching/listing available LLM models
pub struct ModelService<'r> {
    redis: &'r fred::prelude::Pool,
    http_client: &'r reqwest::Client,
}

impl<'r> ModelService<'r> {
    pub fn new(redis: &'r fred::prelude::Pool, http_client: &'r reqwest::Client) -> Self {
        Self { redis, http_client }
    }

    pub async fn list_models(
        &self,
        provider: &ChatRsProvider,
        provider_type: &ChatRsProviderType,
    ) -> Result<Vec<LlmModel>, ModelError> {
        match provider_type {
            ChatRsProviderType::OpenAI => {
                let subtype = provider.openai_subtype.as_deref().unwrap_or_default();
                let md_provider = match OpenAISubtype::from_str(subtype).unwrap_or_default() {
                    OpenAISubtype::OpenAI => ModelsDevProvider::OpenAI,
                    OpenAISubtype::OpenRouter => ModelsDevProvider::OpenRouter,
                };
                self.fetch_models_dev(md_provider).await
            }
            ChatRsProviderType::Anthropic => {
                self.fetch_models_dev(ModelsDevProvider::Anthropic).await
            }
            ChatRsProviderType::Ollama => {
                providers::ollama_models(self.http_client, provider.base_url.as_deref()).await
            }
            ChatRsProviderType::Lorem => Ok(vec![LlmModel {
                id: String::from("lorem"),
                name: String::from("lorem"),
                ..Default::default()
            }]),
        }
    }

    /// Fetch detailed model list from `models.dev` with caching
    async fn fetch_models_dev(
        &self,
        md_provider: ModelsDevProvider,
    ) -> Result<Vec<LlmModel>, ModelError> {
        if let Some(models) = self
            .redis
            .hget::<Option<fred::bytes_utils::Str>, _, _>(CACHE_KEY, md_provider.as_ref())
            .await?
            .and_then(|models| serde_json::from_str(&models).ok())
        {
            Ok(models)
        } else {
            let mut res: ModelsDevResponse = self
                .http_client
                .get(MODELS_DEV_URL)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;

            let mut models: Option<Vec<LlmModel>> = None;
            let mut cache: HashMap<String, String> = HashMap::new();

            for provider in ModelsDevProvider::iter() {
                let provider_models: Vec<LlmModel> = res
                    .remove(provider.as_ref())
                    .ok_or_else(|| ModelError::ModelsDevProviderNotFound(provider.into()))?
                    .models
                    .into_iter()
                    .map(|(_, model)| model)
                    .collect();
                let provider_models_str = serde_json::to_string(&provider_models)?;
                cache.insert(provider.as_ref().to_owned(), provider_models_str);

                if md_provider == provider {
                    models = Some(provider_models);
                }
            }

            let pipeline = self.redis.next().pipeline();
            let _: () = pipeline.hset(CACHE_KEY, cache).await?;
            let _: () = pipeline.expire(CACHE_KEY, CACHE_TTL, None).await?;
            let _: () = pipeline.all().await?;

            Ok(models.unwrap_or_default())
        }
    }
}

/// Represents a provider from `models.dev`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, IntoStaticStr, AsRefStr, EnumIter)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
enum ModelsDevProvider {
    OpenAI,
    OpenRouter,
    Anthropic,
}

/// Main response from `models.dev` - map of providers
type ModelsDevResponse = HashMap<String, ModelsDevProviderData>;

/// Provider data from `models.dev`
#[derive(Debug, Deserialize)]
struct ModelsDevProviderData {
    models: HashMap<String, LlmModel>,
}
