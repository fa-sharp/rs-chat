use crate::{error::AppError, services::auth::encryption::EncryptorError};

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("provider not found")]
    NotFound,
    #[error("missing API key")]
    MissingApiKey,
    #[error("invalid provider type: {0}")]
    InvalidProviderType(#[from] strum::ParseError),
    #[error("error reading/writing API keys: {0}")]
    Encryption(#[from] EncryptorError),
    #[error("database error: {0}")]
    Database(#[from] diesel::result::Error),
}

impl From<ProviderError> for AppError {
    fn from(value: ProviderError) -> Self {
        match value {
            ProviderError::NotFound => Self::not_found("provider not found"),
            ProviderError::MissingApiKey => Self::bad_request("missing API key for this provider"),
            error => Self::internal(error.into()),
        }
    }
}
