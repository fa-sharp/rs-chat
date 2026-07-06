use aide::OperationOutput;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::db::DbPoolError;

/// Global API result type that can be used in route handlers
pub type AppResult<T> = Result<T, AppError>;

/// Global API error type
#[derive(Debug)]
pub struct AppError {
    status: StatusCode,
    message: String,
    source: Option<anyhow::Error>,
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

    pub fn unauthorized(source: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: "unauthorized".into(),
            source: Some(anyhow::anyhow!(source.into())),
        }
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

impl From<diesel::result::Error> for AppError {
    fn from(err: diesel::result::Error) -> Self {
        Self::internal(anyhow::Error::from(err).context("database error"))
    }
}
impl From<DbPoolError> for AppError {
    fn from(err: DbPoolError) -> Self {
        Self::internal(anyhow::Error::from(err).context("database pool error"))
    }
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ErrorBody {
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

impl OperationOutput for AppError {
    type Inner = ErrorResponse;

    fn inferred_responses(
        ctx: &mut aide::generate::GenContext,
        operation: &mut aide::openapi::Operation,
    ) -> Vec<(Option<aide::openapi::StatusCode>, aide::openapi::Response)> {
        if let Some(response) = Json::<ErrorResponse>::operation_response(ctx, operation) {
            let status_codes = [
                StatusCode::BAD_REQUEST,
                StatusCode::UNAUTHORIZED,
                StatusCode::NOT_FOUND,
                StatusCode::INTERNAL_SERVER_ERROR,
            ];
            Vec::from_iter(status_codes.into_iter().map(|code| {
                let aide_code = aide::openapi::StatusCode::Code(code.as_u16());
                (Some(aide_code), response.clone())
            }))
        } else {
            Vec::new()
        }
    }
}
