use rocket::{delete, get, post, serde::json::Json, Route, State};
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
};
use schemars::JsonSchema;
use uuid::Uuid;

use crate::{
    auth::ChatRsUserId,
    db::{
        models::{ChatRsApiKey, ChatRsApiKeyProviderType, NewChatRsApiKey},
        services::api_key::ApiKeyDbService,
        DbConnection,
    },
    errors::ApiError,
    utils::encryption::Encryptor,
};

pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: get_all_api_keys, create_api_key, delete_api_key]
}

/// List all API keys
#[openapi(tag = "API Keys")]
#[get("/")]
async fn get_all_api_keys(
    user_id: ChatRsUserId,
    mut db: DbConnection,
) -> Result<Json<Vec<ChatRsApiKey>>, ApiError> {
    let keys = ApiKeyDbService::new(&mut db)
        .find_by_user_id(&user_id)
        .await?;

    Ok(Json(keys))
}

#[derive(JsonSchema, serde::Deserialize)]
struct ApiKeyInput {
    provider: ChatRsApiKeyProviderType,
    key: String,
}

/// Create a new API key
#[openapi(tag = "API Keys")]
#[post("/", data = "<input>")]
async fn create_api_key(
    user_id: ChatRsUserId,
    mut db: DbConnection,
    encryptor: &State<Encryptor>,
    input: Json<ApiKeyInput>,
) -> Result<String, ApiError> {
    let (ciphertext, nonce) = encryptor.encrypt_string(&input.key)?;
    let id = ApiKeyDbService::new(&mut db)
        .create(NewChatRsApiKey {
            user_id: &user_id,
            provider: &input.provider,
            ciphertext: &ciphertext,
            nonce: &nonce,
        })
        .await?;

    Ok(id.to_string())
}

/// Delete an API key
#[openapi(tag = "API Keys")]
#[delete("/<api_key_id>")]
async fn delete_api_key(
    user_id: ChatRsUserId,
    mut db: DbConnection,
    api_key_id: Uuid,
) -> Result<(), ApiError> {
    let _ = ApiKeyDbService::new(&mut db)
        .delete(&user_id, &api_key_id)
        .await?;

    Ok(())
}
