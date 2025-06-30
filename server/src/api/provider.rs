use fred::prelude::KeysInterface;
use rocket::{get, serde::json::Json, Route, State};
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
};
use schemars::JsonSchema;
use uuid::Uuid;

use crate::{
    db::{
        models::{ChatRsApiKeyProviderType, ChatRsUser},
        services::api_key::ApiKeyDbService,
        DbConnection,
    },
    errors::ApiError,
    redis::RedisClient,
    utils::{
        create_provider::{
            create_provider, LLMBackendInput, LLMConfig, OpenRouterConfig, ProviderConfigInput,
        },
        encryption::Encryptor,
    },
};

pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: get_provider_info]
}

#[derive(JsonSchema, serde::Serialize)]
struct ProviderInfo {
    models: Vec<String>,
}

/// Get provider details
#[openapi(tag = "Provider")]
#[get("/?<provider_type>")]
async fn get_provider_info(
    user: ChatRsUser,
    mut db: DbConnection,
    redis: RedisClient,
    encryptor: &State<Encryptor>,
    provider_type: ChatRsApiKeyProviderType,
) -> Result<Json<ProviderInfo>, ApiError> {
    let cached_models: Option<Vec<String>> = redis
        .get::<Option<String>, _>(cached_models_key(&user.id, &provider_type))
        .await?
        .and_then(|val| serde_json::from_str(&val).ok());
    if let Some(models) = cached_models {
        return Ok(Json(ProviderInfo { models }));
    }

    let provider_config: ProviderConfigInput = match provider_type {
        ChatRsApiKeyProviderType::Anthropic => ProviderConfigInput::Llm(LLMConfig {
            backend: LLMBackendInput::Anthropic,
            ..Default::default()
        }),
        ChatRsApiKeyProviderType::Openai => ProviderConfigInput::Llm(LLMConfig {
            backend: LLMBackendInput::OpenAI,
            ..Default::default()
        }),
        ChatRsApiKeyProviderType::Openrouter => {
            ProviderConfigInput::OpenRouter(OpenRouterConfig::default())
        }
    };

    let provider = create_provider(
        &user.id,
        &provider_config,
        &mut ApiKeyDbService::new(&mut db),
        &encryptor,
    )
    .await?;
    let models = provider.list_models().await?;

    let _: () = redis
        .set(
            cached_models_key(&user.id, &provider_type),
            serde_json::to_string(&models).ok(),
            Some(fred::types::Expiration::EX(60 * 60 * 2)), // 2 hours
            None,
            false,
        )
        .await?;

    Ok(Json(ProviderInfo { models }))
}

fn cached_models_key(user_id: &Uuid, provider: &ChatRsApiKeyProviderType) -> String {
    format!("models:user-{}:{:?}", user_id, provider)
}
