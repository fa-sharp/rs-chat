use rocket::{delete, get, post, serde::json::Json, Route, State};
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
};
use schemars::JsonSchema;
use uuid::Uuid;

use crate::{
    auth::{build_api_key_string, ChatRsUserId},
    db::{
        models::{ChatRsApiKey, NewChatRsApiKey},
        services::ApiKeyDbService,
        DbConnection,
    },
    errors::ApiError,
    utils::Encryptor,
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
struct ApiKeyCreateInput {
    name: String,
}

#[derive(JsonSchema, serde::Serialize)]
struct ApiKeyCreateResponse {
    id: Uuid,
    key: String,
}

/// Create a new API key
#[openapi(tag = "API Keys")]
#[post("/", data = "<input>")]
async fn create_api_key(
    user_id: ChatRsUserId,
    mut db: DbConnection,
    encryptor: &State<Encryptor>,
    input: Json<ApiKeyCreateInput>,
) -> Result<Json<ApiKeyCreateResponse>, ApiError> {
    let key_id = ApiKeyDbService::new(&mut db)
        .create(NewChatRsApiKey {
            user_id: &user_id,
            name: &input.name,
        })
        .await?;
    let (ciphertext, nonce) = encryptor.encrypt_bytes(key_id.as_bytes())?;

    Ok(Json(ApiKeyCreateResponse {
        id: key_id,
        key: build_api_key_string(&ciphertext, &nonce),
    }))
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
