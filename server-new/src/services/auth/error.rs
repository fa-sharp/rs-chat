use crate::{db::DbPoolError, error::AppError};

pub type AuthResult<T> = Result<T, AuthError>;

/// Auth service errors
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("{0}")]
    Unauthorized(&'static str),
    #[error("{0}")]
    BadRequest(&'static str),
    #[error(transparent)]
    Provider(#[from] anyhow::Error),
    #[error("user not found")]
    UserNotFound,
    #[error("database error: {0}")]
    Database(#[from] diesel::result::Error),
    #[error("database error: {0}")]
    DatabasePool(#[from] DbPoolError),
    #[error("session error: {0}")]
    Session(#[from] tower_sessions::session::Error),
    #[error("request error: {0}")]
    Request(#[from] reqwest::Error),
}

// Conversion to HTTP API errors
impl From<AuthError> for AppError {
    fn from(error: AuthError) -> Self {
        match error {
            AuthError::Unauthorized(reason) => Self::unauthorized(reason),
            AuthError::BadRequest(reason) => Self::bad_request(reason),
            AuthError::Provider(error) => Self::internal(error.context("OAuth error")),
            error => Self::internal(error.into()),
        }
    }
}
