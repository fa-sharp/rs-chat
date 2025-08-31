use std::{
    io::Result as IoResult,
    path::{Path, PathBuf},
};
use tokio::{
    fs::File,
    io::{AsyncRead, AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
};
use uuid::Uuid;

pub struct LocalStorage {
    base_path: PathBuf,
}

impl LocalStorage {
    pub fn new(base_path: PathBuf) -> Self {
        LocalStorage { base_path }
    }

    pub async fn read_file(
        &self,
        user_id: &Uuid,
        session_id: Option<&Uuid>,
        path: &Path,
        range: Option<(u64, u64)>,
    ) -> IoResult<Vec<u8>> {
        let path = self.get_file_path(user_id, session_id, path)?;

        let mut file = File::open(path).await?;
        let buffer = if let Some((start, end)) = range {
            let len = (end - start) as usize;
            let mut buffer = vec![0; len];
            file.seek(std::io::SeekFrom::Start(start)).await?;
            file.read(&mut buffer[..len]).await?;
            buffer
        } else {
            let metadata = file.metadata().await?;
            let mut buffer = Vec::with_capacity(metadata.len() as usize);
            file.read_to_end(&mut buffer).await?;
            buffer
        };
        Ok(buffer)
    }

    pub async fn create_file(
        &self,
        user_id: &Uuid,
        session_id: Option<&Uuid>,
        path: &Path,
        mut data: impl AsyncRead + Unpin,
    ) -> IoResult<File> {
        let dir = self.get_user_directory(user_id, session_id);
        tokio::fs::create_dir_all(&dir).await?;

        let file_path = self.get_file_path(user_id, session_id, path)?;
        let mut file = File::create_new(&file_path).await?;

        let mut buffer = [0; 4096];
        while let Ok(n) = data.read(&mut buffer).await {
            if n == 0 {
                break;
            }
            file.write_all(&buffer[..n]).await?;
        }

        file.flush().await?;
        file.sync_all().await?;
        Ok(file)
    }

    fn get_user_directory(&self, user_id: &Uuid, session_id: Option<&Uuid>) -> PathBuf {
        let mut dir = self.base_path.join(user_id.to_string());
        match session_id {
            Some(session_id) => {
                dir.push("sessions");
                dir.push(session_id.to_string());
                dir
            }
            None => {
                dir.push("files");
                dir
            }
        }
    }

    pub fn get_file_path(
        &self,
        user_id: &Uuid,
        session_id: Option<&Uuid>,
        path: &Path,
    ) -> IoResult<PathBuf> {
        if !path.is_relative() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path must be relative",
            ));
        }
        Ok(self.get_user_directory(user_id, session_id).join(path))
    }
}
