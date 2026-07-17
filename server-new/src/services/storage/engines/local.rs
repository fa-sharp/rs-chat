use std::path::{Path, PathBuf};

use futures::future::BoxFuture;
use tokio::{
    fs::File,
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt, BufWriter},
};

use crate::services::storage::{
    StorageEngine,
    error::{StorageError, StorageResult},
};

/// Default storage engine using local filesystem
pub struct LocalStorage {
    base_path: PathBuf,
}

impl LocalStorage {
    pub fn new(base_path: &str) -> Self {
        Self {
            base_path: PathBuf::from(base_path),
        }
    }

    fn file_path(&self, path: &Path) -> PathBuf {
        self.base_path.join(path)
    }

    async fn file_exists(path: &Path) -> bool {
        match tokio::fs::metadata(&path).await {
            Ok(meta) => meta.is_file(),
            Err(_) => false,
        }
    }
}

impl StorageEngine for LocalStorage {
    fn create<'r>(
        &self,
        path: &Path,
        reader: &'r mut (dyn AsyncRead + Unpin + Send),
    ) -> BoxFuture<'r, StorageResult<usize>> {
        let file_path = self.file_path(path);

        Box::pin(async move {
            let dir = file_path.parent().expect("should have a parent directory");
            tokio::fs::create_dir_all(&dir).await?;

            let mut file = File::create_new(&file_path).await?;
            let mut file_writer = BufWriter::new(&mut file);
            let mut read_buffer = [0; 4096];
            let mut total_bytes_written: usize = 0;

            loop {
                let n = reader.read(&mut read_buffer).await?;
                if n == 0 {
                    break;
                }
                file_writer.write_all(&read_buffer[..n]).await?;
                total_bytes_written += n;
            }

            file_writer.flush().await?;
            file.sync_all().await?;

            Ok(total_bytes_written)
        })
    }

    fn exists(&self, path: &Path) -> BoxFuture<'_, StorageResult<bool>> {
        let file_path = self.file_path(path);

        Box::pin(async move { Ok(Self::file_exists(&file_path).await) })
    }

    fn delete(&self, path: &Path) -> BoxFuture<'_, StorageResult<()>> {
        let file_path = self.file_path(path);

        Box::pin(async move {
            match Self::file_exists(&file_path).await {
                true => Ok(tokio::fs::remove_file(&file_path).await?),
                false => Err(StorageError::NotFound),
            }
        })
    }

    fn signed_url(&self, path: &Path) -> StorageResult<String> {
        todo!()
    }
}
