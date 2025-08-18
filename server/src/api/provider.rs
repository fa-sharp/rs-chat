use rocket::{delete, get, patch, post, serde::json::Json, Route, State};
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
};
use schemars::JsonSchema;
use uuid::Uuid;

use crate::{
    auth::ChatRsUserId,
    db::{
        models::{
            ChatRsProvider, ChatRsProviderType, NewChatRsProvider, NewChatRsSecret,
            UpdateChatRsProvider, UpdateChatRsSecret,
        },
        services::{ProviderDbService, SecretDbService},
        DbConnection,
    },
    errors::ApiError,
    provider::build_llm_provider_api,
    provider_models::LlmModel,
    redis::RedisClient,
    utils::Encryptor,
};

pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        settings: get_all_providers,
        list_models,
        create_provider,
        update_provider,
        delete_provider
    ]
}

/// # List providers
/// List all configured providers
#[openapi(tag = "Providers")]
#[get("/")]
async fn get_all_providers(
    user_id: ChatRsUserId,
    mut db: DbConnection,
) -> Result<Json<Vec<ChatRsProvider>>, ApiError> {
    let providers = ProviderDbService::new(&mut db)
        .find_by_user_id(&user_id)
        .await?;

    Ok(Json(providers))
}

/// # List models
/// List all models for a provider
#[openapi(tag = "Providers")]
#[get("/<provider_id>/models")]
async fn list_models(
    user_id: ChatRsUserId,
    mut db: DbConnection,
    redis: RedisClient,
    encryptor: &State<Encryptor>,
    http_client: &State<reqwest::Client>,
    provider_id: i32,
) -> Result<Json<Vec<LlmModel>>, ApiError> {
    let (provider, api_key_secret) = ProviderDbService::new(&mut db)
        .get_by_id(&user_id, provider_id)
        .await?;
    let provider_type: ChatRsProviderType = provider.provider_type.as_str().try_into()?;
    let api_key = api_key_secret
        .map(|secret| encryptor.decrypt_string(&secret.ciphertext, &secret.nonce))
        .transpose()?;
    let provider_api = build_llm_provider_api(
        &provider_type,
        provider.base_url.as_deref(),
        api_key.as_deref(),
        &http_client,
        &redis,
    )?;

    Ok(Json(provider_api.list_models().await?))
}

#[derive(JsonSchema, serde::Deserialize)]
struct ProviderCreateInput {
    name: String,
    r#type: ChatRsProviderType,
    base_url: Option<String>,
    default_model: String,
    api_key: Option<String>,
}

/// # Create provider
/// Create a new LLM provider
#[openapi(tag = "Providers")]
#[post("/", data = "<input>")]
async fn create_provider(
    user_id: ChatRsUserId,
    mut db: DbConnection,
    encryptor: &State<Encryptor>,
    input: Json<ProviderCreateInput>,
) -> Result<Json<ChatRsProvider>, ApiError> {
    let mut api_key_id: Option<Uuid> = None;
    if let Some(plaintext_key) = input.api_key.as_deref() {
        let (ciphertext, nonce) = encryptor.encrypt_string(plaintext_key)?;
        let secret_id = SecretDbService::new(&mut db)
            .create(NewChatRsSecret {
                user_id: &user_id,
                name: &format!("{} API Key", input.name),
                ciphertext: &ciphertext,
                nonce: &nonce,
            })
            .await?;
        api_key_id = Some(secret_id);
    }
    let provider = ProviderDbService::new(&mut db)
        .create(NewChatRsProvider {
            name: &input.name,
            user_id: &user_id,
            provider_type: (&input.r#type).into(),
            base_url: input.base_url.as_deref(),
            default_model: &input.default_model,
            api_key_id,
        })
        .await?;

    Ok(Json(provider))
}

#[derive(JsonSchema, serde::Deserialize)]
struct ProviderUpdateInput {
    name: Option<String>,
    base_url: Option<String>,
    default_model: Option<String>,
    api_key: Option<String>,
}

/// # Update provider
/// Update an LLM Provider
#[openapi(tag = "Providers")]
#[patch("/<provider_id>", data = "<input>")]
async fn update_provider(
    user_id: ChatRsUserId,
    mut db: DbConnection,
    provider_id: i32,
    encryptor: &State<Encryptor>,
    input: Json<ProviderUpdateInput>,
) -> Result<Json<ChatRsProvider>, ApiError> {
    let (provider, secret) = ProviderDbService::new(&mut db)
        .get_by_id(&user_id, provider_id)
        .await?;

    let mut secret_id: Option<Uuid> = None;
    if let Some(new_plaintext_key) = input.api_key.as_deref() {
        let (ciphertext, nonce) = encryptor.encrypt_string(new_plaintext_key)?;
        secret_id = match secret {
            Some(existing_secret) => Some(
                SecretDbService::new(&mut db)
                    .update(
                        &user_id,
                        &existing_secret.id,
                        UpdateChatRsSecret {
                            ciphertext: Some(&ciphertext),
                            nonce: Some(&nonce),
                            ..Default::default()
                        },
                    )
                    .await?,
            ),
            None => Some(
                SecretDbService::new(&mut db)
                    .create(NewChatRsSecret {
                        user_id: &user_id,
                        name: &format!("{} API Key", provider.name),
                        ciphertext: &ciphertext,
                        nonce: &nonce,
                    })
                    .await?,
            ),
        };
    }

    let updated = ProviderDbService::new(&mut db)
        .update(
            &user_id,
            provider_id,
            UpdateChatRsProvider {
                api_key_id: secret_id,
                name: input.name.as_deref(),
                base_url: input.base_url.as_deref(),
                default_model: input.default_model.as_deref(),
            },
        )
        .await?;

    Ok(Json(updated))
}

/// # Delete provider
/// Delete an LLM Provider
#[openapi(tag = "Providers")]
#[delete("/<provider_id>")]
async fn delete_provider(
    user_id: ChatRsUserId,
    mut db: DbConnection,
    provider_id: i32,
) -> Result<Json<ChatRsProvider>, ApiError> {
    let (provider, api_key_secret) = ProviderDbService::new(&mut db)
        .get_by_id(&user_id, provider_id)
        .await?;
    if let Some(secret) = api_key_secret {
        SecretDbService::new(&mut db)
            .delete(&user_id, &secret.id)
            .await?;
    }
    ProviderDbService::new(&mut db)
        .delete(&user_id, provider_id)
        .await?;

    Ok(Json(provider))
}
