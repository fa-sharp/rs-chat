use std::path::{Path, PathBuf};

use futures::{FutureExt, future::BoxFuture};
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
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    fn local_path(&self, file_path: &Path) -> PathBuf {
        self.base_path.join(file_path)
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
        &'r self,
        file_path: &'r Path,
        reader: &'r mut (dyn AsyncRead + Unpin + Send),
    ) -> BoxFuture<'r, StorageResult<usize>> {
        Box::pin(async move {
            let local_path = self.local_path(file_path);
            let dir = local_path.parent().expect("should always have parent dir");
            tokio::fs::create_dir_all(&dir).await?;

            let mut file = File::create_new(&local_path).await?;
            let mut file_writer = BufWriter::new(&mut file);
            let mut read_buffer = [0; 8192];
            let mut total_bytes: usize = 0;

            while let n = reader.read(&mut read_buffer).await?
                && n != 0
            {
                file_writer.write_all(&read_buffer[..n]).await?;
                total_bytes += n;
            }

            file_writer.flush().await?;
            file.sync_all().await?;

            Ok(total_bytes)
        })
    }

    fn exists<'r>(&'r self, file_path: &'r Path) -> BoxFuture<'r, StorageResult<bool>> {
        async move { Ok(Self::file_exists(&self.local_path(file_path)).await) }.boxed()
    }

    fn delete<'r>(&'r self, file_path: &'r Path) -> BoxFuture<'r, StorageResult<()>> {
        Box::pin(async move {
            let local_path = self.local_path(file_path);

            match Self::file_exists(&local_path).await {
                true => Ok(tokio::fs::remove_file(&local_path).await?),
                false => Err(StorageError::NotFound),
            }
        })
    }
}
