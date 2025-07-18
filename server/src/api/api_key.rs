use rocket::{delete, get, post, serde::json::Json, Route, State};
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
};
use schemars::JsonSchema;
use uuid::Uuid;

use crate::{
    auth::ChatRsUserId,
    db::{
        models::{ChatRsProviderKeyType, ChatRsSecretMeta, NewChatRsSecret},
        services::SecretDbService,
        DbConnection,
    },
    errors::ApiError,
    utils::encryption::Encryptor,
};

pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: get_all_provider_keys, create_provider_key, delete_provider_key]
}

/// List all Provider API keys
#[openapi(tag = "Secrets")]
#[get("/")]
async fn get_all_provider_keys(
    user_id: ChatRsUserId,
    mut db: DbConnection,
) -> Result<Json<Vec<ChatRsSecretMeta>>, ApiError> {
    let keys = SecretDbService::new(&mut db)
        .find_by_user_id(&user_id)
        .await?;

    Ok(Json(keys))
}

#[derive(JsonSchema, serde::Deserialize)]
struct ProviderKeyInput {
    provider: ChatRsProviderKeyType,
    key: String,
    name: String,
}

/// Create a new Provider API key
#[openapi(tag = "Secrets")]
#[post("/", data = "<input>")]
async fn create_provider_key(
    user_id: ChatRsUserId,
    mut db: DbConnection,
    encryptor: &State<Encryptor>,
    input: Json<ProviderKeyInput>,
) -> Result<String, ApiError> {
    let (ciphertext, nonce) = encryptor.encrypt_string(&input.key)?;
    let id = SecretDbService::new(&mut db)
        .create(NewChatRsSecret {
            user_id: &user_id,
            name: &input.name,
            provider: &input.provider,
            ciphertext: &ciphertext,
            nonce: &nonce,
        })
        .await?;

    Ok(id.to_string())
}

/// Delete a Provider API key
#[openapi(tag = "Secrets")]
#[delete("/<api_key_id>")]
async fn delete_provider_key(
    user_id: ChatRsUserId,
    mut db: DbConnection,
    api_key_id: Uuid,
) -> Result<(), ApiError> {
    let _ = SecretDbService::new(&mut db)
        .delete(&user_id, &api_key_id)
        .await?;

    Ok(())
}
