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
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    fn local_path(&self, file_path: &Path) -> PathBuf {
        self.base_path.join(file_path)
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
        Box::pin(async move {
            let exists = tokio::fs::try_exists(&self.local_path(file_path)).await?;

            Ok(exists)
        })
    }

    fn delete<'r>(&'r self, file_path: &'r Path) -> BoxFuture<'r, StorageResult<()>> {
        Box::pin(async move {
            let local_path = self.local_path(file_path);
            if tokio::fs::try_exists(&local_path).await? {
                return Err(StorageError::NotFound);
            }

            tokio::fs::remove_file(&local_path).await?;

            // Clean up parent directories
            let mut parent_dir = local_path.clone();
            while let Some(dir) = parent_dir.parent().filter(|dir| *dir != self.base_path) {
                match tokio::fs::remove_dir(dir).await {
                    Ok(_) => parent_dir = dir.to_path_buf(),
                    Err(err) if err.kind() == std::io::ErrorKind::DirectoryNotEmpty => {
                        break;
                    }
                    Err(err) => return Err(StorageError::Io(err)),
                };
            }

            Ok(())
        })
    }
}
