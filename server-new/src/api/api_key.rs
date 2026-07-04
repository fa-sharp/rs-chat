use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    api::ApiTag,
    db::models::ChatRsApiKey,
    error::AppResult,
    extractors::{CurrentUser, Database},
    state::AppState,
};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(list_api_keys, create_api_key, delete_api_key))
}

/// List all API keys
#[utoipa::path(
    get, path = "",
    responses((status = OK, body = Vec<ChatRsApiKey>)),
    tag = ApiTag::ApiKey.into())
]
async fn list_api_keys(
    CurrentUser { user_id }: CurrentUser,
    Database(mut db): Database,
) -> AppResult<Json<Vec<ChatRsApiKey>>> {
    let keys = db.api_keys().find_by_user_id(&user_id).await?;
    Ok(Json(keys))
}

#[derive(Deserialize, ToSchema)]
struct ApiKeyCreateInput {
    name: String,
}

#[derive(Serialize, ToSchema)]
struct ApiKeyCreateResponse {
    id: Uuid,
    key: String,
}

/// Create an API key
#[utoipa::path(
    post, path = "",
    request_body = ApiKeyCreateInput,
    responses((status = OK, body = ApiKeyCreateResponse)),
    tag = ApiTag::ApiKey.into())
]
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

/// Delete an API key
#[utoipa::path(
    delete, path = "/{id}",
    params(("id" = Uuid, Path)),
    responses((status = NO_CONTENT)),
    tag = ApiTag::ApiKey.into())
]
async fn delete_api_key(
    CurrentUser { user_id }: CurrentUser,
    Path(api_key_id): Path<Uuid>,
    Database(mut db): Database,
) -> AppResult<StatusCode> {
    let _deleted_id = db.api_keys().delete(&user_id, &api_key_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
