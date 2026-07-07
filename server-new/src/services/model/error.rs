use crate::error::AppError;

#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("provider not supported")]
    ProviderNotSupported,
    #[error("invalid provider type: {0}")]
    InvalidProviderType(#[from] strum::ParseError),
    #[error("models.dev request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("models.dev provider not found: {0}")]
    ProviderNotFound(&'static str),
    #[error("Redis error: {0}")]
    Redis(#[from] fred::prelude::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl From<ModelError> for AppError {
    fn from(error: ModelError) -> Self {
        match error {
            ModelError::ProviderNotSupported => Self::bad_request("listing models not supported"),
            err => Self::internal(err.into()),
        }
    }
}
