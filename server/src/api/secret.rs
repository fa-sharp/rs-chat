use rocket::{delete, get, post, serde::json::Json, Route, State};
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
};
use schemars::JsonSchema;
use uuid::Uuid;

use crate::{
    auth::ChatRsUserId,
    db::{
        models::{ChatRsSecretMeta, NewChatRsSecret},
        services::SecretDbService,
        DbConnection,
    },
    errors::ApiError,
    utils::Encryptor,
};

pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: get_all_secrets, create_secret, delete_secret]
}

/// List all secrets
#[openapi(tag = "Secrets")]
#[get("/")]
async fn get_all_secrets(
    user_id: ChatRsUserId,
    mut db: DbConnection,
) -> Result<Json<Vec<ChatRsSecretMeta>>, ApiError> {
    let secrets = SecretDbService::new(&mut db)
        .find_by_user_id(&user_id)
        .await?;

    Ok(Json(secrets))
}

#[derive(JsonSchema, serde::Deserialize)]
pub struct SecretInput {
    pub key: String,
    pub name: String,
}

/// Create a new secret
#[openapi(tag = "Secrets")]
#[post("/", data = "<input>")]
async fn create_secret(
    user_id: ChatRsUserId,
    mut db: DbConnection,
    encryptor: &State<Encryptor>,
    input: Json<SecretInput>,
) -> Result<String, ApiError> {
    let (ciphertext, nonce) = encryptor.encrypt_string(&input.key)?;
    let id = SecretDbService::new(&mut db)
        .create(NewChatRsSecret {
            user_id: &user_id,
            name: &input.name,
            ciphertext: &ciphertext,
            nonce: &nonce,
        })
        .await?;

    Ok(id.to_string())
}

/// Delete a secret
#[openapi(tag = "Secrets")]
#[delete("/<secret_id>")]
async fn delete_secret(
    user_id: ChatRsUserId,
    mut db: DbConnection,
    secret_id: Uuid,
) -> Result<(), ApiError> {
    let _ = SecretDbService::new(&mut db)
        .delete(&user_id, &secret_id)
        .await?;

    Ok(())
}
