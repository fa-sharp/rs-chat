use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    db::{
        models::{ChatRsFileType, NewChatRsFile},
        services::FileDbService,
        DbConnection,
    },
    provider::LlmToolType,
    storage::{LocalStorage, DEFAULT_DATA_DIR},
};

use super::*;

/// Tools for listing, reading, and writing files in the current chat session.
pub struct Files<'a> {
    user_id: &'a uuid::Uuid,
    session_id: &'a uuid::Uuid,
    app_config: &'a AppConfig,
    db: &'a mut DbConnection,
    config: &'a FilesConfig,
}
impl<'a> Files<'a> {
    pub fn new(
        user_id: &'a uuid::Uuid,
        session_id: &'a uuid::Uuid,
        app_config: &'a AppConfig,
        db: &'a mut DbConnection,
        config: &'a FilesConfig,
    ) -> Self {
        Self {
            user_id,
            session_id,
            app_config,
            db,
            config,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct FilesConfig {
    storage: StorageType,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
enum StorageType {
    Local,
}

#[derive(Debug, Default, PartialEq, JsonSchema, Serialize, Deserialize)]
pub struct FilesInput {
    /// Whether assistant has permission to read files
    read: bool,
    /// Whether assistant has permission to write files
    write: bool,
}

const LIST_FILES: &str = "list_files";
const LIST_FILES_DESC: &str = "List files for the current chat session";
static LIST_FILES_SCHEMA: LazyLock<serde_json::Value> =
    LazyLock::new(|| utils::get_json_schema::<ListFilesInput>());

const READ_FILE: &str = "read_file";
const READ_FILE_DESC: &str = "Read a file in the current chat session";
static READ_FILE_SCHEMA: LazyLock<serde_json::Value> =
    LazyLock::new(|| utils::get_json_schema::<ReadFileInput>());

const WRITE_FILE: &str = "write_file";
const WRITE_FILE_DESC: &str = "Write a file in the current chat session";
static WRITE_FILE_SCHEMA: LazyLock<serde_json::Value> =
    LazyLock::new(|| utils::get_json_schema::<WriteFileInput>());

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ListFilesInput {
    /// Optional directory path to list files from
    #[schemars(example = "dir_path_example")]
    dir: Option<String>,
}
fn dir_path_example() -> &'static str {
    "foo/dir"
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ReadFileInput {
    /// Path of the file to read. Should be a relative path.
    #[schemars(example = "file_path_example")]
    path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct WriteFileInput {
    /// Path of the file to write. Should be a relative path.
    #[schemars(example = "file_path_example")]
    path: String,
    /// Content to write to the file
    content: String,
}
fn file_path_example() -> &'static str {
    "foo/file.txt"
}

impl SystemToolConfig for FilesConfig {
    type DynamicConfig = FilesInput;

    fn get_llm_tools(
        &self,
        tool_id: uuid::Uuid,
        input_config: Option<&Self::DynamicConfig>,
    ) -> Vec<LlmTool> {
        let mut tools = Vec::with_capacity(3);

        if input_config.map_or(true, |c| c.read) {
            tools.push(LlmTool {
                tool_id,
                name: READ_FILE.into(),
                description: READ_FILE_DESC.into(),
                input_schema: READ_FILE_SCHEMA.to_owned(),
                tool_type: LlmToolType::System,
            });
            tools.push(LlmTool {
                tool_id,
                name: LIST_FILES.into(),
                description: LIST_FILES_DESC.into(),
                input_schema: LIST_FILES_SCHEMA.to_owned(),
                tool_type: LlmToolType::System,
            });
        }
        if input_config.map_or(true, |c| c.write) {
            tools.push(LlmTool {
                tool_id,
                name: WRITE_FILE.into(),
                description: WRITE_FILE_DESC.into(),
                input_schema: WRITE_FILE_SCHEMA.to_owned(),
                tool_type: LlmToolType::System,
            });
        }

        tools
    }

    fn validate(&self) -> ToolResult<()> {
        Ok(())
    }
}

#[async_trait]
impl SystemTool for Files<'_> {
    fn input_schema(&self, tool_name: &str) -> ToolResult<&serde_json::Value> {
        match tool_name {
            READ_FILE => Ok(&READ_FILE_SCHEMA),
            LIST_FILES => Ok(&LIST_FILES_SCHEMA),
            WRITE_FILE => Ok(&WRITE_FILE_SCHEMA),
            _ => Err(ToolError::ToolNotFound),
        }
    }

    async fn execute(
        &mut self,
        tool_name: &str,
        parameters: serde_json::Value,
        _sender: &SenderWithLogging<ToolLog>,
    ) -> ToolResult<(String, ToolResponseFormat)> {
        let storage = match self.config.storage {
            StorageType::Local => {
                let data_dir = self
                    .app_config
                    .data_dir
                    .as_deref()
                    .unwrap_or(DEFAULT_DATA_DIR);
                LocalStorage::new(PathBuf::from(data_dir).join("storage"))
            }
        };

        match tool_name {
            READ_FILE => {
                let input: ReadFileInput = serde_json::from_value(parameters)?;
                let path = Path::new(&input.path);
                let content_bytes = storage
                    .read_file(self.user_id, Some(self.session_id), path)
                    .await?;
                let content = String::from_utf8_lossy(&content_bytes);
                Ok((content.into(), ToolResponseFormat::Text))
            }
            LIST_FILES => {
                let input: ListFilesInput =
                    serde_json::from_value(serde_json::to_value(parameters)?)?;
                let mut files = FileDbService::new(self.db)
                    .list_session_files(self.user_id, self.session_id)
                    .await?;
                if let Some(dir) = input.dir {
                    files.retain(|file| file.path.starts_with(&dir));
                }

                Ok((serde_json::to_string(&files)?, ToolResponseFormat::Json))
            }
            WRITE_FILE => {
                let input: WriteFileInput =
                    serde_json::from_value(serde_json::to_value(parameters)?)?;
                let size = storage
                    .create_file(
                        self.user_id,
                        Some(self.session_id),
                        &Path::new(&input.path),
                        input.content.as_bytes(),
                    )
                    .await?;
                let file = FileDbService::new(self.db)
                    .create_session_file(NewChatRsFile {
                        user_id: self.user_id,
                        session_id: Some(self.session_id),
                        path: &input.path,
                        file_type: ChatRsFileType::Text.into(),
                        content_type: "text/plain",
                        size: size.try_into().unwrap_or_default(),
                    })
                    .await?;

                let message = format!("File '{}' created with ID {}", input.path, file.id);
                Ok((message, ToolResponseFormat::Text))
            }
            _ => Err(ToolError::ToolNotFound),
        }
    }
}
