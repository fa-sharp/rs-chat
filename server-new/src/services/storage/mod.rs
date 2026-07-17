use std::path::PathBuf;

use axum::extract::multipart::MultipartError;
use futures::{Stream, TryStreamExt};
use tokio_util::io::StreamReader;
use uuid::Uuid;

use crate::services::storage::{engines::LocalStorage, error::StorageResult};

mod engines;
mod error;
mod interface;

pub use interface::StorageEngine;

pub struct StorageService<'r> {
    data_dir: &'r str,
}

impl<'r> StorageService<'r> {
    fn file_path(&self, user_id: &Uuid, session_id: &Uuid, name: &str) -> PathBuf {
        let session_folder = PathBuf::from(format!("{user_id}/{session_id}"));

        session_folder.join(name)
    }

    pub async fn create_file(
        &self,
        user_id: &Uuid,
        session_id: &Uuid,
        name: &str,
        stream: impl Stream<Item = Result<axum::body::Bytes, MultipartError>> + Send + Unpin,
    ) -> StorageResult<()> {
        let storage: Box<dyn StorageEngine> =
            Box::new(LocalStorage::new(&format!("{}/storage", self.data_dir)));

        let path = self.file_path(user_id, session_id, name);
        let mut reader = StreamReader::new(stream.map_err(std::io::Error::other));

        let size = match storage.create(&path, &mut reader).await {
            Ok(n) => n,
            Err(err) => {
                let _ = storage.delete(&path).await;
                return Err(err);
            }
        };

        todo!()
    }
}
