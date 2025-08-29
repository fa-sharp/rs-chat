use std::collections::HashMap;

use enum_iterator::{all, Sequence};
use fred::prelude::{HashesInterface, KeysInterface};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::provider::LlmError;

const CACHE_KEY: &str = "models";
const CACHE_TTL: i64 = 86400; // 1 day in seconds

/// A model supported by the LLM provider
#[derive(Debug, Default, Clone, JsonSchema, Serialize, Deserialize)]
pub struct LlmModel {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub knowledge: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modalities: Option<Modalities>,
    // Ollama fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
}

#[derive(Debug, Clone, JsonSchema, Serialize, Deserialize)]
pub struct Modalities {
    input: Vec<ModalityType>,
    output: Vec<ModalityType>,
}

#[derive(Debug, Clone, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModalityType {
    Text,
    Image,
    Audio,
    Video,
    Pdf,
}

/// Service to fetch and cache LLM model list from https://models.dev
pub struct ModelsDevService {
    redis: fred::clients::Client,
    http_client: reqwest::Client,
}

impl ModelsDevService {
    pub fn new(redis: &fred::clients::Client, http_client: &reqwest::Client) -> Self {
        Self {
            redis: redis.clone(),
            http_client: http_client.clone(),
        }
    }

    pub async fn list_models(
        &self,
        provider: ModelsDevServiceProvider,
    ) -> Result<Vec<LlmModel>, LlmError> {
        if let Some(models) = self
            .redis
            .hget::<Option<String>, _, &str>(CACHE_KEY, (&provider).into())
            .await?
            .and_then(|models| serde_json::from_str(&models).ok())
        {
            Ok(models)
        } else {
            let mut res: ModelsDevResponse = self
                .http_client
                .get("https://models.dev/api.json")
                .send()
                .await
                .map_err(|e| LlmError::ModelsDevError(e.to_string()))?
                .json()
                .await
                .map_err(|e| LlmError::ModelsDevError(e.to_string()))?;

            let mut models: Option<Vec<LlmModel>> = None;
            let mut cache: HashMap<String, String> = HashMap::new();
            for model_provider in all::<ModelsDevServiceProvider>() {
                let provider_str: &str = (&model_provider).into();
                let provider_response = res.remove(provider_str).ok_or_else(|| {
                    LlmError::ModelsDevError(format!("Provider '{}' not found", provider_str))
                })?;
                let parsed_models: Vec<LlmModel> = provider_response
                    .models
                    .into_iter()
                    .map(|(_, model)| model)
                    .collect();
                let parsed_models_str = serde_json::to_string(&parsed_models)
                    .map_err(|e| LlmError::ModelsDevError(e.to_string()))?;

                if provider == model_provider {
                    models = Some(parsed_models);
                }
                cache.insert(provider_str.to_owned(), parsed_models_str);
            }

            let pipeline = self.redis.pipeline();
            let _: () = pipeline.hset(CACHE_KEY, cache).await?;
            let _: () = pipeline.expire(CACHE_KEY, CACHE_TTL, None).await?;
            let _: () = pipeline.all().await?;

            Ok(models.unwrap_or_default())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Sequence)]
#[serde(rename_all = "lowercase")]
pub enum ModelsDevServiceProvider {
    OpenAI,
    Anthropic,
    OpenRouter,
}
impl From<&ModelsDevServiceProvider> for &'static str {
    fn from(provider: &ModelsDevServiceProvider) -> Self {
        match provider {
            ModelsDevServiceProvider::OpenAI => "openai",
            ModelsDevServiceProvider::Anthropic => "anthropic",
            ModelsDevServiceProvider::OpenRouter => "openrouter",
        }
    }
}

/// The response from the models.dev API.
type ModelsDevResponse = HashMap<String, ModelsDevProviderResponse>;

#[derive(Debug, Deserialize)]
struct ModelsDevProviderResponse {
    models: HashMap<String, LlmModel>,
}
