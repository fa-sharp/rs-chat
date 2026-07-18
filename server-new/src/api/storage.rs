use axum::{
    Json,
    extract::{Path, State},
};
use axum_aide_macros::api_routes;
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    api::ApiTag,
    db::models::{ChatRsFile, ChatRsMessageAttachment},
    error::AppResult,
    extractors::{CurrentUser, Database},
    state::AppState,
};

api_routes! {
    state: AppState,
    tag: ApiTag::Storage.into(),
    GET "/user" => list_user_files, "List user files";
    DELETE "/user/{file_id}" => delete_user_file, "Delete user file";
    GET "/session/{session_id}" => list_session_files, "List session files";
    DELETE "/session/{session_id}/{file_id}" => delete_session_file, "Delete session file";
}

async fn list_user_files(
    CurrentUser { user_id }: CurrentUser,
    Database(mut db): Database,
) -> AppResult<Json<Vec<ChatRsFile>>> {
    let files = db.files().list_user_files(&user_id).await?;

    Ok(Json(files))
}

async fn list_session_files(
    CurrentUser { user_id }: CurrentUser,
    Path(session_id): Path<Uuid>,
    Database(mut db): Database,
) -> AppResult<Json<SessionFilesAndAttachments>> {
    let (files, attachments) = db
        .files()
        .list_session_files_and_attachments(&user_id, &session_id)
        .await?;

    Ok(Json(SessionFilesAndAttachments { files, attachments }))
}

async fn delete_user_file(
    CurrentUser { user_id }: CurrentUser,
    Path(file_id): Path<Uuid>,
    Database(mut db): Database,
    State(state): State<AppState>,
) -> AppResult<Json<FileIdResponse>> {
    let file_id = state
        .storage_service()
        .delete_file(&mut db, &user_id, None, &file_id)
        .await?;

    Ok(Json(FileIdResponse { file_id }))
}

async fn delete_session_file(
    CurrentUser { user_id }: CurrentUser,
    Path((session_id, file_id)): Path<(Uuid, Uuid)>,
    Database(mut db): Database,
    State(state): State<AppState>,
) -> AppResult<Json<FileIdResponse>> {
    let file_id = state
        .storage_service()
        .delete_file(&mut db, &user_id, Some(&session_id), &file_id)
        .await?;

    Ok(Json(FileIdResponse { file_id }))
}

#[derive(Serialize, JsonSchema)]
struct SessionFilesAndAttachments {
    files: Vec<ChatRsFile>,
    attachments: Vec<ChatRsMessageAttachment>,
}

#[derive(Serialize, JsonSchema)]
struct FileIdResponse {
    file_id: Uuid,
}
