use aide::OperationIo;
use axum::{RequestExt, extract::FromRequest, http::header};
use futures::{Stream, TryStreamExt};

use crate::{error::AppError, state::AppState};

/// Extractor to get a streaming uploaded file
#[derive(OperationIo)]
pub struct FileUpload {
    body: axum::body::Body,
    content_type: String,
    content_length: usize,
}

impl FromRequest<AppState> for FileUpload {
    type Rejection = AppError;

    async fn from_request(
        req: axum::extract::Request,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let content_type = req
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| AppError::bad_request("no content-type header"))?
            .to_owned();
        let content_length: usize = req
            .headers()
            .get(header::CONTENT_LENGTH)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.parse().ok())
            .ok_or_else(|| AppError::bad_request("no content-length header"))?;

        let body = req.into_limited_body();

        Ok(Self {
            body,
            content_length,
            content_type,
        })
    }
}

impl FileUpload {
    pub fn content_type(&self) -> String {
        self.content_type.to_owned()
    }

    pub fn size(&self) -> usize {
        self.content_length
    }

    pub fn into_stream(self) -> impl Stream<Item = Result<axum::body::Bytes, std::io::Error>> {
        self.body.into_data_stream().map_err(std::io::Error::other)
    }
}
