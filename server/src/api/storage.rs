use std::path::{Path, PathBuf};

use rocket::{fs::NamedFile, get, post, Route, State};
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
};

use crate::{
    auth::ChatRsUserId,
    errors::ApiError,
    storage::{FileData, LocalStorage},
};

pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: upload_file, download_file]
}

/// Upload a new file
#[openapi(tag = "Files")]
#[post("/?<path>", data = "<file>")]
async fn upload_file(
    user_id: ChatRsUserId,
    storage: &State<LocalStorage>,
    path: &str,
    // mut db: DbConnection,
    file: FileData<'_>,
) -> Result<(), ApiError> {
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
    storage: &State<LocalStorage>,
    path: PathBuf,
) -> Result<NamedFile, ApiError> {
    let file_path = storage.get_file_path(&user_id, None, &path)?;
    let file = NamedFile::open(file_path).await?;

    Ok(file)
}
