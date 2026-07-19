use std::path::{Path, PathBuf};

use futures::Stream;
use uuid::Uuid;

use crate::{
    config::StorageConfig,
    db::{
        DbPool, DbService,
        models::{ChatRsFile, ChatRsFileType, NewChatRsFile},
    },
    services::storage::{
        engines::{LocalStorage, S3Storage},
        error::{StorageError, StorageResult},
    },
};

pub mod engines;
mod error;
mod interface;

pub use interface::StorageEngine;

/// Name of the base folder containing all files/attachments
pub const STORAGE_FOLDER: &str = "rs-chat/storage";
/// Name of the folder containing user files
pub const USER_FOLDER: &str = "user";
/// Name of the folder containing session files
pub const SESSION_FOLDER: &str = "session";

pub struct StorageService<'r> {
    data_dir: &'r Path,
    db_pool: &'r DbPool,
    config: &'r StorageConfig,
    http_client: &'r reqwest::Client,
}

impl<'r> StorageService<'r> {
    pub fn new(
        data_dir: &'r Path,
        db_pool: &'r DbPool,
        http_client: &'r reqwest::Client,
        config: &'r StorageConfig,
    ) -> Self {
        Self {
            data_dir,
            db_pool,
            config,
            http_client,
        }
    }

    pub async fn create_file(
        &self,
        user_id: &Uuid,
        session_id: Option<&Uuid>,
        path: &str,
        size: usize,
        content_type: &str,
        stream: impl Stream<Item = Result<axum::body::Bytes, std::io::Error>> + Send + 'static,
    ) -> StorageResult<ChatRsFile> {
        let file_path = self.build_file_path(user_id, session_id, &path)?;
        let file_type = self.validate_file_type(content_type)?;

        let storage = self.storage_engine()?;
        if storage.exists(&file_path).await? {
            return Err(StorageError::AlreadyExists);
        }

        match storage
            .create(&file_path, size, content_type, Box::pin(stream))
            .await
        {
            Ok(bytes_written) if bytes_written == size => {}
            Ok(wrong_size) => {
                let _ = storage.delete(&file_path).await;
                return Err(StorageError::WrongSize(wrong_size));
            }
            Err(err) => {
                let _ = storage.delete(&file_path).await;
                return Err(err);
            }
        };

        let mut db = DbService::from_pool(self.db_pool).await?;
        let new_file = NewChatRsFile {
            user_id,
            session_id,
            path,
            file_type: file_type.as_ref(),
            content_type,
            size: size.try_into().unwrap_or_default(),
        };
        let db_file = db.files().create_file(new_file).await?;

        Ok(db_file)
    }

    pub async fn delete_file(
        &self,
        db: &mut DbService,
        user_id: &Uuid,
        session_id: Option<&Uuid>,
        file_id: &Uuid,
    ) -> StorageResult<Uuid> {
        let db_file = match session_id {
            Some(session_id) => {
                db.files()
                    .find_session_file(user_id, session_id, file_id)
                    .await?
            }
            None => db.files().find_user_file(user_id, file_id).await?,
        }
        .ok_or(StorageError::NotFound)?;

        let storage = self.storage_engine()?;
        let file_path = self.build_file_path(user_id, session_id, &db_file.path)?;
        if let Err(err) = storage.delete(&file_path).await {
            tracing::warn!("error deleting file {file_id} with path {file_path:?}: {err}");
        }

        let deleted_file_id = match session_id {
            Some(session_id) => {
                db.files()
                    .delete_session_file(user_id, session_id, file_id)
                    .await?
            }
            None => db.files().delete_user_file(user_id, file_id).await?,
        };

        Ok(deleted_file_id)
    }

    fn storage_engine(&self) -> StorageResult<Box<dyn StorageEngine + 'r>> {
        Ok(match self.config {
            StorageConfig::Local => Box::new(LocalStorage::new(self.data_dir.join(STORAGE_FOLDER))),
            StorageConfig::S3(config) => Box::new(S3Storage::new(
                STORAGE_FOLDER.into(),
                config,
                self.http_client,
            )?),
        })
    }

    fn build_file_path(
        &self,
        user_id: &Uuid,
        session_id: Option<&Uuid>,
        path: &str,
    ) -> StorageResult<PathBuf> {
        let valid_path = Path::new(path).is_relative()
            && Path::new(path)
                .components()
                .all(|c| matches!(c, std::path::Component::Normal(_)));
        if !valid_path {
            return Err(StorageError::InvalidPath(path.into()));
        }

        Ok(match session_id {
            Some(session_id) => {
                let segments = [
                    &user_id.to_string(),
                    SESSION_FOLDER,
                    &session_id.to_string(),
                    &path,
                ];
                segments.iter().collect()
            }
            None => [&user_id.to_string(), USER_FOLDER, &path].iter().collect(),
        })
    }

    fn validate_file_type(&self, content_type: &str) -> StorageResult<ChatRsFileType> {
        match content_type {
            "image/jpeg" | "image/png" | "image/webp" => Ok(ChatRsFileType::Image),
            "application/pdf" => Ok(ChatRsFileType::Pdf),
            "application/json" | "application/xml" => Ok(ChatRsFileType::Text),
            text if text.starts_with("text/") => Ok(ChatRsFileType::Text),
            unsupported => Err(StorageError::UnsupportedContentType(unsupported.into())),
        }
    }
}
