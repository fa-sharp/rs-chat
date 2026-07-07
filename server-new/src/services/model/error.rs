use crate::error::AppError;

#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("models.dev provider not found: {0}")]
    ModelsDevProviderNotFound(&'static str),
    #[error("Redis error: {0}")]
    Redis(#[from] fred::prelude::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl From<ModelError> for AppError {
    fn from(error: ModelError) -> Self {
        Self::internal(error.into())
    }
}
