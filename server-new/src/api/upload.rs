use axum::{
    Json,
    extract::{Multipart, Path, State},
};
use axum_aide_macros::api_routes;
use futures::TryStreamExt;
use uuid::Uuid;

use crate::{
    api::ApiTag,
    db::models::ChatRsFile,
    error::{AppError, AppResult},
    extractors::{CurrentUser, Database},
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
    mut multipart: Multipart,
) -> AppResult<Json<ChatRsFile>> {
    let field = multipart
        .next_field()
        .await
        .map_err(|err| AppError::bad_request(err.body_text()))?
        .ok_or_else(|| AppError::bad_request("no file in request"))?;
    let mime = field.content_type().map(str::to_owned);
    let stream = field.map_err(std::io::Error::other);

    let file = state
        .storage_service()
        .create_file(&user_id, None, &path, mime.as_deref(), stream)
        .await?;

    Ok(Json(file))
}

async fn upload_session_file(
    CurrentUser { user_id }: CurrentUser,
    Path((sess_id, path)): Path<(Uuid, String)>,
    Database(mut db): Database,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> AppResult<Json<ChatRsFile>> {
    if db.chats().find_session(&user_id, &sess_id).await?.is_none() {
        return Err(AppError::not_found("session not found"));
    }
    drop(db); // free database connection since this could be long-running request

    let field = multipart
        .next_field()
        .await
        .map_err(|err| AppError::bad_request(err.body_text()))?
        .ok_or_else(|| AppError::bad_request("no file in request"))?;
    let mime = field.content_type().map(str::to_owned);
    let stream = field.map_err(std::io::Error::other);

    let file = state
        .storage_service()
        .create_file(&user_id, Some(&sess_id), &path, mime.as_deref(), stream)
        .await?;

    Ok(Json(file))
}
