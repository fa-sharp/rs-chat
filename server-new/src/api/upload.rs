use axum::{
    Json,
    extract::{Path, State},
};
use axum_aide_macros::api_routes;
use uuid::Uuid;

use crate::{
    api::ApiTag,
    db::models::ChatRsFile,
    error::{AppError, AppResult},
    extractors::{CurrentUser, Database, FileUpload},
    state::AppState,
};

api_routes! {
    state: AppState,
    tag: ApiTag::Storage.into(),
    POST "/user/{*file_path}" => upload_user_file, "Upload user file" {
        description: "Upload a file to the user account. The file must be the only field in the form,
        with a supported content type."
    };
    POST "/session/{session_id}/{*file_path}" => upload_session_file, "Upload session file" {
        description: "Upload a file to a chat session. The file must be the only field in the form,
        with a supported content type."
    };
}

async fn upload_user_file(
    CurrentUser { user_id }: CurrentUser,
    Path(path): Path<String>,
    State(state): State<AppState>,
    upload: FileUpload,
) -> AppResult<Json<ChatRsFile>> {
    let file = state
        .storage_service()
        .create_file(
            &user_id,
            None,
            &path,
            upload.size(),
            &upload.content_type(),
            upload.into_stream(),
        )
        .await?;

    Ok(Json(file))
}

async fn upload_session_file(
    CurrentUser { user_id }: CurrentUser,
    Path((sess_id, path)): Path<(Uuid, String)>,
    Database(mut db): Database,
    State(state): State<AppState>,
    upload: FileUpload,
) -> AppResult<Json<ChatRsFile>> {
    if db.chats().find_session(&user_id, &sess_id).await?.is_none() {
        return Err(AppError::not_found("session not found"));
    }
    drop(db); // free database connection since this could be long-running request

    let file = state
        .storage_service()
        .create_file(
            &user_id,
            Some(&sess_id),
            &path,
            upload.size(),
            &upload.content_type(),
            upload.into_stream(),
        )
        .await?;

    Ok(Json(file))
}
