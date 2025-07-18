use rocket::{delete, get, post, serde::json::Json, Route, State};
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
};
use schemars::JsonSchema;
use uuid::Uuid;

use crate::{
    auth::ChatRsUserId,
    db::{
        models::{ChatRsProviderKeyMeta, ChatRsProviderKeyType, NewChatRsProviderKey},
        services::ProviderKeyDbService,
        DbConnection,
    },
    errors::ApiError,
    utils::encryption::Encryptor,
};

pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: get_all_provider_keys, create_provider_key, delete_provider_key]
}

/// List all Provider API keys
#[openapi(tag = "Provider Keys")]
#[get("/")]
async fn get_all_provider_keys(
    user_id: ChatRsUserId,
    mut db: DbConnection,
) -> Result<Json<Vec<ChatRsProviderKeyMeta>>, ApiError> {
    let keys = ProviderKeyDbService::new(&mut db)
        .find_by_user_id(&user_id)
        .await?;

    Ok(Json(keys))
}

#[derive(JsonSchema, serde::Deserialize)]
struct ProviderKeyInput {
    provider: ChatRsProviderKeyType,
    key: String,
}

/// Create a new Provider API key
#[openapi(tag = "Provider Keys")]
#[post("/", data = "<input>")]
async fn create_provider_key(
    user_id: ChatRsUserId,
    mut db: DbConnection,
    encryptor: &State<Encryptor>,
    input: Json<ProviderKeyInput>,
) -> Result<String, ApiError> {
    let (ciphertext, nonce) = encryptor.encrypt_string(&input.key)?;
    let id = ProviderKeyDbService::new(&mut db)
        .create(NewChatRsProviderKey {
            user_id: &user_id,
            provider: &input.provider,
            ciphertext: &ciphertext,
            nonce: &nonce,
        })
        .await?;

    Ok(id.to_string())
}

/// Delete a Provider API key
#[openapi(tag = "Provider Keys")]
#[delete("/<api_key_id>")]
async fn delete_provider_key(
    user_id: ChatRsUserId,
    mut db: DbConnection,
    api_key_id: Uuid,
) -> Result<(), ApiError> {
    let _ = ProviderKeyDbService::new(&mut db)
        .delete(&user_id, &api_key_id)
        .await?;

    Ok(())
}
