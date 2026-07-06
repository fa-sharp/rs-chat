use aide::axum::{
    ApiRouter,
    routing::{delete_with, get_with, post_with},
};
use aide_docs_macro::docs;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    api::ApiTag,
    db::models::ChatRsApiKey,
    error::{AppError, AppResult},
    extractors::{CurrentUser, Database},
    state::AppState,
};

pub fn routes() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", get_with(list_api_keys, list_api_keys_docs))
        .api_route("/", post_with(create_api_key, create_api_key_docs))
        .api_route("/{id}", delete_with(delete_api_key, delete_api_key_docs))
        .with_path_items(|op| op.tag(ApiTag::ApiKey.into()))
}

#[docs("List API keys")]
async fn list_api_keys(
    CurrentUser { user_id }: CurrentUser,
    Database(mut db): Database,
) -> AppResult<Json<Vec<ChatRsApiKey>>> {
    let keys = db.api_keys().find_by_user_id(&user_id).await?;
    Ok(Json(keys))
}

#[derive(Deserialize, JsonSchema)]
struct ApiKeyCreateInput {
    name: String,
}

#[docs("Create API key")]
async fn create_api_key(
    CurrentUser { user_id }: CurrentUser,
    Database(mut db): Database,
    State(state): State<AppState>,
    input: Json<ApiKeyCreateInput>,
) -> AppResult<Json<ApiKeyCreateResponse>> {
    let (id, key) = state
        .auth_service()
        .api_keys()
        .create_api_key(&mut db, &user_id, &input.name)
        .await?;
    Ok(Json(ApiKeyCreateResponse { id, key }))
}

#[derive(Serialize, JsonSchema)]
struct ApiKeyCreateResponse {
    id: Uuid,
    key: String,
}

#[docs("Delete API key")]
async fn delete_api_key(
    Path(id): Path<Uuid>,
    CurrentUser { user_id }: CurrentUser,
    Database(mut db): Database,
) -> AppResult<StatusCode> {
    match db.api_keys().delete(&user_id, &id).await? {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err(AppError::not_found("API key not found")),
    }
}
