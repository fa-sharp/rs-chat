use std::path::PathBuf;

use rocket::{delete, fs::NamedFile, get, post, serde::json::Json, Route, State};
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
};
use uuid::Uuid;

use crate::{
    auth::ChatRsUserId,
    db::{
        models::{ChatRsFile, NewChatRsFile},
        services::FileDbService,
        DbConnection,
    },
    errors::ApiError,
    storage::{FileData, LocalStorage},
};

pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: upload_file, download_file, list_session_files, delete_file]
}

/// List session files
#[openapi(tag = "Storage")]
#[get("/<session_id>")]
async fn list_session_files(
    user_id: ChatRsUserId,
    session_id: Uuid,
    mut db: DbConnection,
) -> Result<Json<Vec<ChatRsFile>>, ApiError> {
    let files = FileDbService::new(&mut db)
        .list_session_files(&user_id, &session_id)
        .await?;

    Ok(Json(files))
}

/// Upload a new session file
#[openapi(tag = "Storage")]
#[post("/<session_id>/<path..>", data = "<file>")]
async fn upload_file(
    user_id: ChatRsUserId,
    session_id: Uuid,
    path: PathBuf,
    file: FileData<'_>,
    storage: &State<LocalStorage>,
    mut db: DbConnection,
) -> Result<Json<ChatRsFile>, ApiError> {
    let size = storage
        .create_file(&user_id, Some(&session_id), &path, file.data)
        .await?;
    let db_file = FileDbService::new(&mut db)
        .create_session_file(NewChatRsFile {
            user_id: &user_id,
            session_id: Some(&session_id),
            path: &path.to_string_lossy(),
            file_type: file.file_type.into(),
            content_type: &file.content_type.to_string(),
            size: size.try_into().unwrap_or_default(),
        })
        .await?;

    Ok(Json(db_file))
}

/// Download a session file
#[openapi(tag = "Storage")]
#[get("/<session_id>/<file_id>")]
async fn download_file(
    user_id: ChatRsUserId,
    session_id: Uuid,
    file_id: Uuid,
    storage: &State<LocalStorage>,
    mut db: DbConnection,
) -> Result<NamedFile, ApiError> {
    let file = FileDbService::new(&mut db)
        .find_session_file(&user_id, &session_id, &file_id)
        .await?;
    let file_path = storage.get_file_path(&user_id, Some(&session_id), &file.path)?;

    Ok(NamedFile::open(file_path).await?)
}

/// Delete a session file
#[openapi(tag = "Storage")]
#[delete("/<session_id>/<file_id>")]
async fn delete_file(
    user_id: ChatRsUserId,
    session_id: Uuid,
    file_id: Uuid,
    storage: &State<LocalStorage>,
    mut db: DbConnection,
) -> Result<String, ApiError> {
    let mut db_service = FileDbService::new(&mut db);
    let file = db_service
        .find_session_file(&user_id, &session_id, &file_id)
        .await?;

    storage
        .delete_file(&user_id, Some(&session_id), &file.path)
        .await?;
    db_service
        .delete_session_file(&user_id, &session_id, &file_id)
        .await?;

    Ok(file_id.to_string())
}
