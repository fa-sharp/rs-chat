use rocket::{get, serde::json::Json, Route, State};
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
};
use schemars::JsonSchema;

use crate::{
    db::{
        models::{ChatRsApiKeyProviderType, ChatRsUser},
        services::api_key::ApiKeyDbService,
        DbConnection,
    },
    errors::ApiError,
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

#[openapi(tag = "Provider")]
#[get("/?<provider>")]
async fn get_provider_info(
    user: ChatRsUser,
    mut db: DbConnection,
    encryptor: &State<Encryptor>,
    provider: ChatRsApiKeyProviderType,
) -> Result<Json<ProviderInfo>, ApiError> {
    let provider_config: ProviderConfigInput = match provider {
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
        _ => unimplemented!(),
    };

    let provider = create_provider(
        &user.id,
        &provider_config,
        &mut ApiKeyDbService::new(&mut db),
        &encryptor,
    )
    .await?;
    let models = provider.list_models().await?;

    Ok(Json(ProviderInfo { models }))
}
