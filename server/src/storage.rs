mod data_guard;
mod local;

use std::path::{Path, PathBuf};

use rocket::fairing::AdHoc;
use uuid::Uuid;

use crate::{
    config::get_app_config,
    db::models::{ChatRsFile, ChatRsFileType},
    provider::LlmError,
};
pub use data_guard::*;
pub use local::*;

/// Default data directory path.
pub const DEFAULT_DATA_DIR: &str = "/data";

/// Setup file reading and writing for the Rocket application.
pub fn setup_storage() -> AdHoc {
    AdHoc::on_ignite("Storage", |rocket| async {
        let app_config = get_app_config(&rocket);
        let data_dir = app_config.data_dir.as_deref().unwrap_or(DEFAULT_DATA_DIR);
        let storage_path = PathBuf::from(data_dir).join("storage");
        let storage = LocalStorage::new(storage_path);

        rocket.manage(storage)
    })
}

impl ChatRsFile {
    /// Get the file type and contents for LLM input. Uses base64 URLs for image and PDF files.
    pub async fn read_to_string(
        &self,
        session_id: Option<&Uuid>,
        storage: &LocalStorage,
    ) -> Result<(ChatRsFileType, String), LlmError> {
        let file_type = ChatRsFileType::try_from(self.file_type.as_str())?;
        let content: String = match file_type {
            ChatRsFileType::Text => {
                let bytes = storage
                    .read_file_as_bytes(&self.user_id, session_id, Path::new(&self.path))
                    .await?;
                String::from_utf8_lossy(&bytes).into()
            }
            ChatRsFileType::Image | ChatRsFileType::Pdf => {
                storage
                    .read_file_as_base64(&self.user_id, session_id, Path::new(&self.path))
                    .await?
            }
        };
        Ok((file_type, content))
    }
}
