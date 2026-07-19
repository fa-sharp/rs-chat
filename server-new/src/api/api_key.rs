use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use axum_aide_macros::api_routes;
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

api_routes! {
    state: AppState,
    tag: ApiTag::ApiKey.into(),
    GET "/" => list_api_keys, "List API keys";
    POST "/" => create_api_key, "Create API key";
    DELETE "/{id}" => delete_api_key, "Delete API key";
}

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
