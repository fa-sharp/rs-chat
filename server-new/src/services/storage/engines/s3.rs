use std::{
    path::{Path, PathBuf},
    sync::{Arc, atomic::AtomicUsize},
    time::Duration,
};

use futures::{TryStreamExt, future::BoxFuture, stream::BoxStream};
use reqwest::{StatusCode, header};
use rusty_s3::{Bucket, Credentials, S3Action};
use serde::{Deserialize, Serialize};

use crate::services::storage::{
    StorageEngine,
    error::{StorageError, StorageResult},
};

/// Expiration used for S3 requests & presigned URLs
const EXPIRY: Duration = Duration::from_secs(60 * 5);

/// S3 storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Config {
    endpoint: reqwest::Url,
    bucket: String,
    region: String,
    access_key: String,
    secret_key: String,
}

/// Storage engine using S3
pub struct S3Storage<'c> {
    base_path: PathBuf,
    bucket: Bucket,
    credentials: Credentials,
    client: &'c reqwest::Client,
}

impl<'c> S3Storage<'c> {
    pub fn new(
        base_path: PathBuf,
        config: &'c S3Config,
        client: &'c reqwest::Client,
    ) -> StorageResult<Self> {
        let bucket = Bucket::new(
            config.endpoint.clone(),
            rusty_s3::UrlStyle::VirtualHost,
            config.bucket.clone(),
            config.region.clone(),
        )
        .map_err(|e| StorageError::Setup(e.to_string()))?;
        let credentials = Credentials::new(&config.access_key, &config.secret_key);

        Ok(Self {
            base_path,
            bucket,
            credentials,
            client,
        })
    }

    fn file_key(&self, path: &Path) -> Result<String, StorageError> {
        let file_path = self.base_path.join(path);
        let file_key = file_path
            .to_str()
            .ok_or_else(|| StorageError::InvalidPath(file_path.to_string_lossy().into_owned()))?;

        Ok(file_key.to_owned())
    }

    async fn handle_response_error(&self, response: reqwest::Response) -> StorageError {
        StorageError::Response(format!(
            "Status: {}, Response: {:?}",
            response.status().as_u16(),
            response.text().await
        ))
    }
}

impl StorageEngine for S3Storage<'_> {
    fn create<'r>(
        &'r self,
        path: &'r Path,
        size: usize,
        content_type: &'r str,
        stream: BoxStream<'static, Result<axum::body::Bytes, std::io::Error>>,
    ) -> BoxFuture<'r, StorageResult<usize>> {
        Box::pin(async move {
            let file_key = self.file_key(path)?;
            let put_object = self.bucket.put_object(Some(&self.credentials), &file_key);
            let url = put_object.sign(EXPIRY);

            let total_bytes = Arc::new(AtomicUsize::new(0));
            let size_counter = Arc::clone(&total_bytes);

            let response = self
                .client
                .put(url)
                .header(header::CONTENT_LENGTH, size)
                .header(header::CONTENT_TYPE, content_type)
                .body(reqwest::Body::wrap_stream(stream.inspect_ok(
                    move |chunk| {
                        size_counter.fetch_add(chunk.len(), std::sync::atomic::Ordering::Relaxed);
                    },
                )))
                .send()
                .await?;

            if !response.status().is_success() {
                return Err(self.handle_response_error(response).await);
            }

            Ok(total_bytes.load(std::sync::atomic::Ordering::Relaxed))
        })
    }

    fn exists<'r>(&'r self, path: &'r Path) -> BoxFuture<'r, StorageResult<bool>> {
        Box::pin(async move {
            let file_key = self.file_key(path)?;
            let head_object = self.bucket.head_object(Some(&self.credentials), &file_key);
            let url = head_object.sign(EXPIRY);

            let response = self.client.head(url).send().await?;
            match response.status() {
                StatusCode::OK => Ok(true),
                StatusCode::NOT_FOUND => Ok(false),
                _ => Err(self.handle_response_error(response).await),
            }
        })
    }

    fn delete<'r>(&'r self, path: &'r Path) -> BoxFuture<'r, StorageResult<()>> {
        Box::pin(async move {
            let file_key = self.file_key(path)?;
            let delete_object = self
                .bucket
                .delete_object(Some(&self.credentials), &file_key);
            let url = delete_object.sign(EXPIRY);

            let response = self.client.delete(url).send().await?;
            match response.status() {
                StatusCode::NO_CONTENT => Ok(()),
                _ => Err(self.handle_response_error(response).await),
            }
        })
    }

    fn signed_url(&self, path: &Path) -> StorageResult<Option<String>> {
        let file_key = self.file_key(path)?;
        let get_object = self.bucket.get_object(Some(&self.credentials), &file_key);
        let url = get_object.sign(EXPIRY);

        Ok(Some(url.into()))
    }
}
