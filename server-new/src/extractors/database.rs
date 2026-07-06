use aide::OperationIo;
use axum::extract::FromRequestParts;

use crate::{db::DbService, error::AppError, state::AppState};

/// An extractor to retrieve a database connection from the pool
#[derive(OperationIo)]
pub struct Database(pub DbService);

impl FromRequestParts<AppState> for Database {
    type Rejection = AppError;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match DbService::from_pool(&state.db_pool).await {
            Ok(db_service) => Ok(Self(db_service)),
            Err(err) => Err(AppError::internal(err.into())),
        }
    }
}
