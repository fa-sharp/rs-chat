use std::path::PathBuf;

use rocket::{fs::TempFile, post, serde::json::Json, Route, State};
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
};
use uuid::Uuid;

use crate::{
    config::AppConfig,
    db::{
        models::{
            ChatRsFile, ChatRsFileContentType, ChatRsFileStorageType, ChatRsUser, NewChatRsFile,
        },
        services::storage::StorageDbService,
        DbConnection,
    },
    errors::ApiError,
    web::WEB_DIST,
};

pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        settings: upload_file
    ]
}

/// Upload a file
#[openapi(tag = "Storage")]
#[post("/upload?<name>&<type>", data = "<file>")]
async fn upload_file(
    user: ChatRsUser,
    mut db: DbConnection,
    app_config: &State<AppConfig>,
    mut file: TempFile<'_>,
    name: &str,
    r#type: ChatRsFileContentType,
) -> Result<Json<ChatRsFile>, ApiError> {
    // Get user's upload folder path, and ensure the folder exists
    let static_folder =
        tokio::fs::canonicalize(app_config.static_path.as_deref().unwrap_or(WEB_DIST)).await?;
    let file_folder = static_folder.join(PathBuf::from(format!("uploads/{}", user.id)));
    tokio::fs::create_dir_all(&file_folder).await?;

    // Save the file to the upload folder
    let file_id = Uuid::new_v4();
    let file_path = file_folder.join(PathBuf::from(file_id.to_string()));
    file.persist_to(&file_path).await?;

    // Save in database
    let saved_file = StorageDbService::new(&mut db)
        .create(NewChatRsFile {
            id: &file_id,
            user_id: &user.id,
            name,
            content_type: &r#type,
            storage: &ChatRsFileStorageType::Local,
            path: file_path.to_str().ok_or(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Invalid file path",
            ))?,
        })
        .await?;

    Ok(Json(saved_file))
}
