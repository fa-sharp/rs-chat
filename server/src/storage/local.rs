use std::{
    io::Result as IoResult,
    path::{Path, PathBuf},
};
use tokio::{
    fs::File,
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
};
use uuid::Uuid;

pub struct LocalStorage {
    base_path: PathBuf,
}

impl LocalStorage {
    pub fn new(base_path: PathBuf) -> Self {
        LocalStorage { base_path }
    }

    pub async fn read_file_as_bytes(
        &self,
        user_id: &Uuid,
        session_id: Option<&Uuid>,
        path: &Path,
    ) -> IoResult<Vec<u8>> {
        let path = self.get_file_path(user_id, session_id, path)?;
        let mut file = File::open(path).await?;
        let metadata = file.metadata().await?;
        let mut file_reader = BufReader::new(&mut file);

        let mut buffer = Vec::with_capacity(metadata.len() as usize);
        file_reader.read_to_end(&mut buffer).await?;
        Ok(buffer)
    }

    pub async fn read_file_as_base64(
        &self,
        user_id: &Uuid,
        session_id: Option<&Uuid>,
        path: &Path,
    ) -> IoResult<String> {
        let path = self.get_file_path(user_id, session_id, path)?;
        tokio::task::spawn_blocking(move || read_base64(&path)).await?
    }

    pub async fn create_file(
        &self,
        user_id: &Uuid,
        session_id: Option<&Uuid>,
        path: &Path,
        mut data: impl AsyncRead + Unpin,
    ) -> IoResult<usize> {
        let file_path = self.get_file_path(user_id, session_id, path)?;
        let dir = file_path.parent().expect("Should have a parent directory");
        tokio::fs::create_dir_all(&dir).await?;

        let mut file = File::create_new(&file_path).await?;
        let mut file_writer = BufWriter::new(&mut file);
        let mut read_buffer = [0; 4096];
        let mut total_bytes_written: usize = 0;
        while let Ok(n) = data.read(&mut read_buffer).await {
            if n == 0 {
                break;
            }
            file_writer.write_all(&read_buffer[..n]).await?;
            total_bytes_written += n;
        }

        file_writer.flush().await?;
        file.sync_all().await?;

        Ok(total_bytes_written)
    }

    pub async fn delete_file<P: AsRef<Path>>(
        &self,
        user_id: &Uuid,
        session_id: Option<&Uuid>,
        path: P,
    ) -> IoResult<()> {
        let file_path = self.get_file_path(user_id, session_id, path)?;
        tokio::fs::remove_file(&file_path).await
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

    pub fn get_file_path<P: AsRef<Path>>(
        &self,
        user_id: &Uuid,
        session_id: Option<&Uuid>,
        path: P,
    ) -> IoResult<PathBuf> {
        if !path.as_ref().is_relative() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path must be relative",
            ));
        }
        Ok(self.get_user_directory(user_id, session_id).join(path))
    }
}

/// Reads a file as a base64 encoded string (synchronous because `base64` crate writer is synchronous).
fn read_base64(path: &Path) -> IoResult<String> {
    let mut file = std::fs::File::open(path)?;
    let file_size = file.metadata()?.len();
    let estimated_size = (file_size + 2) / 3 * 4;
    let mut file_reader = std::io::BufReader::new(&mut file);

    let mut result = Vec::with_capacity(estimated_size as usize);
    {
        let mut encoder = base64::write::EncoderWriter::new(
            &mut result,
            &base64::engine::general_purpose::STANDARD,
        );
        std::io::copy(&mut file_reader, &mut encoder)?;
        encoder.finish()?;
    }
    Ok(String::from_utf8(result).expect("base64 is valid UTF8"))
}
