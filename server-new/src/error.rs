use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

/// Global result type that can be used for API route handlers
pub type AppResult<T> = Result<T, AppError>;

/// Global error type
#[derive(Debug)]
pub struct AppError {
    status: StatusCode,
    message: String,
    source: Option<anyhow::Error>,
}

impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        Self::internal(error)
    }
}

impl From<diesel::result::Error> for AppError {
    fn from(error: diesel::result::Error) -> Self {
        Self::internal(error.into())
    }
}

impl AppError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
            source: None,
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    pub fn unauthorized() -> Self {
        Self::new(StatusCode::UNAUTHORIZED, "unauthorized")
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    pub fn internal(error: anyhow::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "internal server error".to_string(),
            source: Some(error),
        }
    }
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    message: String,
    status: u16,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        if let Some(error) = self.source {
            tracing::warn!(error = ?error, "request failed");
        }

        let response = ErrorResponse {
            error: ErrorBody {
                message: self.message,
                status: self.status.as_u16(),
            },
        };

        (self.status, Json(response)).into_response()
    }
}
