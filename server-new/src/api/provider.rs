use aide_docs_macro::api_routes;
use axum::{
    Json,
    extract::{Path, State},
};

use crate::{
    api::ApiTag,
    db::models::ChatRsProvider,
    error::AppResult,
    extractors::{CurrentUser, Database},
    services::{
        model::types::LlmModel,
        provider::types::{ProviderCreateInput, ProviderUpdateInput},
    },
    state::AppState,
};

api_routes! {
    state: AppState,
    tag: ApiTag::Provider,
    GET "/" => list_providers, "List providers";
    GET "/{id}/models" => list_models, "List models";
    POST "/" => create_provider, "Create provider";
    PATCH "/{id}" => update_provider, "Update provider";
    DELETE "/{id}" => delete_provider, "Delete provider";
}

async fn list_providers(
    CurrentUser { user_id }: CurrentUser,
    Database(mut db): Database,
) -> AppResult<Json<Vec<ChatRsProvider>>> {
    let providers = db.providers().list_by_user_id(&user_id).await?;
    Ok(Json(providers))
}

async fn list_models(
    CurrentUser { user_id }: CurrentUser,
    Path(provider_id): Path<i32>,
    Database(mut db): Database,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<LlmModel>>> {
    let (provider, provider_type, _) = state
        .provider_service()
        .get_provider(&mut db, &user_id, provider_id)
        .await?;
    let models = state
        .model_service()
        .list_models(&provider, &provider_type)
        .await?;

    Ok(Json(models))
}

async fn create_provider(
    CurrentUser { user_id }: CurrentUser,
    Database(mut db): Database,
    State(state): State<AppState>,
    Json(input): Json<ProviderCreateInput>,
) -> AppResult<Json<ChatRsProvider>> {
    let provider = state
        .provider_service()
        .create_provider(&mut db, &user_id, &input)
        .await?;
    Ok(Json(provider))
}

async fn update_provider(
    CurrentUser { user_id }: CurrentUser,
    Path(provider_id): Path<i32>,
    Database(mut db): Database,
    State(state): State<AppState>,
    Json(input): Json<ProviderUpdateInput>,
) -> AppResult<Json<ChatRsProvider>> {
    let updated_provider = state
        .provider_service()
        .update_provider(&mut db, &user_id, provider_id, &input)
        .await?;
    Ok(Json(updated_provider))
}

async fn delete_provider(
    CurrentUser { user_id }: CurrentUser,
    Path(provider_id): Path<i32>,
    Database(mut db): Database,
    State(state): State<AppState>,
) -> AppResult<Json<ChatRsProvider>> {
    let deleted_provider = state
        .provider_service()
        .delete_provider(&mut db, &user_id, provider_id)
        .await?;
    Ok(Json(deleted_provider))
}
