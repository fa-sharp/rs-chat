//! Storage operation errors

use crate::{db::DbPoolError, error::AppError};

pub type StorageResult<T> = Result<T, StorageError>;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("File not found")]
    NotFound,
    #[error("File with this path already exists")]
    AlreadyExists,
    #[error("Unsupported content type: '{0}'")]
    UnsupportedContentType(String),
    #[error("Invalid file name/path: '{0}'")]
    InvalidPath(String),
    #[error("File had unexpected size: {0} bytes")]
    WrongSize(usize),

    #[error("Storage setup error: {0}")]
    Setup(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Storage request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Storage response error: {0}")]
    Response(String),
    #[error("Database error: {0}")]
    Database(#[from] diesel::result::Error),
    #[error(transparent)]
    DatabasePool(#[from] DbPoolError),
}

impl From<StorageError> for AppError {
    fn from(error: StorageError) -> Self {
        match error {
            StorageError::NotFound => AppError::not_found(error.to_string()),
            StorageError::AlreadyExists
            | StorageError::UnsupportedContentType(_)
            | StorageError::InvalidPath(_) => Self::bad_request(error.to_string()),
            err => Self::internal(err.into()),
        }
    }
}
