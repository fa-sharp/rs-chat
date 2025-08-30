use std::path::{Path, PathBuf};

use rocket::{fs::NamedFile, get, post, Route, State};
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
};

use crate::{
    auth::ChatRsUserId,
    config::AppConfig,
    errors::ApiError,
    storage::{FileData, LocalStorage},
};

pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: upload_file, download_file]
}

const DEFAULT_DATA_DIR: &str = "/data";
fn get_storage_path(app_config: &AppConfig) -> PathBuf {
    let data_dir = app_config.data_dir.as_deref().unwrap_or(DEFAULT_DATA_DIR);
    PathBuf::from(data_dir).join("storage")
}

/// Upload a new file
#[openapi(tag = "Files")]
#[post("/?<path>", data = "<file>")]
async fn upload_file(
    user_id: ChatRsUserId,
    app_config: &State<AppConfig>,
    path: &str,
    // mut db: DbConnection,
    file: FileData<'_>,
) -> Result<(), ApiError> {
    let storage = LocalStorage::new(get_storage_path(app_config));
    let path = Path::new(path);

    let new_file = storage
        .create_file(&user_id, None, &path, file.data)
        .await?;
    println!("File created successfully: {:?}", new_file);

    Ok(())
}

/// Download a file
#[openapi(tag = "Files")]
#[get("/<path>")]
async fn download_file(
    user_id: ChatRsUserId,
    app_config: &State<AppConfig>,
    path: PathBuf,
) -> Result<NamedFile, ApiError> {
    let storage = LocalStorage::new(get_storage_path(app_config));
    let file_path = storage.get_file_path(&user_id, None, &path)?;
    let file = NamedFile::open(file_path).await?;

    Ok(file)
}
