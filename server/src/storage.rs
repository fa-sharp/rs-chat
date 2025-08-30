use rocket::{
    async_trait,
    data::{FromData, Outcome, ToByteUnit},
    http::{ContentType, Status},
    outcome::{try_outcome, IntoOutcome},
    Request,
};
use rocket_okapi::request::OpenApiFromData;
use std::{
    io::Result as IoResult,
    path::{Path, PathBuf},
};
use tokio::{
    fs::File,
    io::{AsyncRead, AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
};
use uuid::Uuid;

const MAX_FILE_SIZE: usize = 4 * 1024 * 1024; // 4 MB

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

/// Data guard for file uploads
pub struct FileData<'r> {
    pub data: rocket::data::DataStream<'r>,
    pub content_type: &'r ContentType,
    pub file_type: FileType,
    pub content_length: usize,
}

/// File modality
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum FileType {
    Text,
    Image,
    Pdf,
}

#[async_trait]
impl<'r> FromData<'r> for FileData<'r> {
    type Error = &'static str;

    async fn from_data(
        req: &'r Request<'_>,
        mut data: rocket::Data<'r>,
    ) -> Outcome<'r, Self, Self::Error> {
        if data.peek(8).await.is_empty() {
            return Outcome::Error((Status::BadRequest, "No data found"));
        }
        let content_type = try_outcome!(req
            .content_type()
            .or_error((Status::BadRequest, "No content type found")));
        let content_length: usize = try_outcome!(req
            .headers()
            .get_one("Content-Length")
            .map(|s| s.parse().unwrap_or(0))
            .or_error((Status::LengthRequired, "No content length found")));
        if content_length > MAX_FILE_SIZE {
            return Outcome::Error((Status::PayloadTooLarge, "File size exceeds maximum"));
        }

        let file_type = {
            if content_type.is_jpeg()
                || content_type.is_png()
                || content_type.is_webp()
                || content_type.is_bmp()
            {
                FileType::Image
            } else if content_type.is_pdf() {
                FileType::Pdf
            } else {
                FileType::Text
            }
        };

        Outcome::Success(FileData {
            data: data.open(5.mebibytes()),
            file_type,
            content_length,
            content_type,
        })
    }
}

impl<'r> OpenApiFromData<'r> for FileData<'r> {
    fn request_body(
        _gen: &mut rocket_okapi::r#gen::OpenApiGenerator,
    ) -> rocket_okapi::Result<rocket_okapi::okapi::openapi3::RequestBody> {
        Ok(rocket_okapi::okapi::openapi3::RequestBody {
            description: Some("File data".to_string()),
            content: {
                let mut content = schemars::Map::new();
                content.insert(
                    "application/octet-stream".into(),
                    rocket_okapi::okapi::openapi3::MediaType {
                        schema: Some(rocket_okapi::okapi::openapi3::SchemaObject {
                            instance_type: Some(schemars::schema::SingleOrVec::Single(Box::new(
                                schemars::schema::InstanceType::String,
                            ))),
                            format: Some("binary".to_string()),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                );
                content
            },
            required: true,
            ..Default::default()
        })
    }
}
