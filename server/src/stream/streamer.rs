use rocket::futures::StreamExt;
use tinistream_client::types::StreamAccessResponse;
use uuid::Uuid;

use crate::{
    db::{
        models::{
            AssistantMeta, ChatRsFileType, ChatRsMessageMeta, ChatRsMessageRole, NewChatRsFile,
            NewChatRsMessage,
        },
        services::{ChatDbService, FileDbService},
        DbConnection,
    },
    errors::ApiError,
    provider::{LlmProviderOptions, LlmStream},
    storage::LocalStorage,
    stream::TinistreamClient,
};

/// Utility that handles streaming to clients and persisting responses from the provider
pub struct LlmClientStreamer {
    db: DbConnection,
    tinistream: TinistreamClient,
    storage: LocalStorage,
}

impl LlmClientStreamer {
    pub fn new(db: DbConnection, tinistream: &TinistreamClient, storage: &LocalStorage) -> Self {
        Self {
            db,
            tinistream: tinistream.to_owned(),
            storage: storage.to_owned(),
        }
    }

    pub async fn start(
        mut self,
        stream: LlmStream,
        stream_key: String,
        user_id: Uuid,
        session_id: Uuid,
        provider_id: i32,
        provider_options: LlmProviderOptions,
    ) -> Result<StreamAccessResponse, ApiError> {
        // Create the Redis stream in `tinistream` and get a WebSocket connection for writing to it
        let stream_access = self.tinistream.stream_start(&stream_key).await?;
        let (ws_writer, ws_reader) = self.tinistream.stream_writer_ws(&stream_key).await?.split();

        // Spawn a task to finish streaming and process/save the response
        tokio::spawn(async move {
            let response = super::LlmStreamWriter::new()
                .process(stream, ws_writer, ws_reader)
                .await;

            // Save generated images
            let mut image_ids: Option<Vec<Uuid>> = None;
            for image in response.images.unwrap_or_default() {
                let path = format!("generated/{}.png", Uuid::new_v4());
                match self
                    .storage
                    .create_file_from_data_url(&user_id, Some(&session_id), &path, image.base64_url)
                    .await
                {
                    Ok((content_type, size)) => {
                        match FileDbService::new(&mut self.db)
                            .create_session_file(NewChatRsFile {
                                user_id: &user_id,
                                session_id: Some(&session_id),
                                path: &path,
                                file_type: ChatRsFileType::Image.into(),
                                content_type: &content_type,
                                size: size.try_into().unwrap_or_default(),
                            })
                            .await
                        {
                            Ok(file) => image_ids.get_or_insert_default().push(file.id),
                            Err(err) => rocket::error!("Failed to save image to db: {err}"),
                        }
                    }
                    Err(err) => rocket::error!("Failed to save image to storage: {err}"),
                }
            }

            // Save response message and metadata
            let assistant_meta = AssistantMeta {
                provider_id,
                provider_options: Some(provider_options),
                tool_calls: response.tool_calls,
                files: image_ids,
                usage: response.usage,
                errors: response.errors,
                partial: response.cancelled.then_some(true),
            };
            if let Err(err) = ChatDbService::new(&mut self.db)
                .save_message(NewChatRsMessage {
                    session_id: &session_id,
                    role: ChatRsMessageRole::Assistant,
                    content: &response.text.unwrap_or_default(),
                    meta: ChatRsMessageMeta::new_assistant(assistant_meta),
                })
                .await
            {
                rocket::error!("Failed to save assistant message: {err}");
            }

            // Signal end of stream
            if !response.cancelled {
                self.tinistream.stream_end(&stream_key).await.ok();
            }
        });

        Ok(stream_access)
    }
}
