use std::sync::Arc;

use tinistream_client::types::{StreamAccessResponse, StreamStatus};
use uuid::Uuid;

use crate::{
    db::{
        DbPool, DbService,
        models::{
            AssistantMeta, ChatRsLogKind, ChatRsLogStatus, ChatRsMessage, ChatRsMessageMeta,
            ChatRsMessageRole, NewChatRsMessage, UserMeta,
        },
    },
    llm::{
        interface::{LlmProvider, LlmResponseMeta},
        types::{LlmChatOptions, LlmChatRequest, LlmMessage, LlmUserMessage},
    },
    services::{
        chat::error::ChatError,
        stream::{LlmStreamOutput, StreamingService, tinistream::TinistreamClient},
    },
};

mod error;
mod messages;
mod titles;

pub const DEFAULT_SESSION_TITLE: &str = "New Chat";

pub struct ChatService<'r> {
    db_pool: &'r DbPool,
    tinistream: &'r TinistreamClient,
}

#[derive(Debug)]
struct ChatStreamParams {
    user_id: Uuid,
    session_id: Uuid,
    provider_id: i32,
    chat_options: LlmChatOptions,
    replace_message_id: Option<Uuid>,
}

impl<'r> ChatService<'r> {
    pub fn new(db_pool: &'r DbPool, tinistream: &'r TinistreamClient) -> Self {
        Self {
            db_pool,
            tinistream,
        }
    }

    /// Connect to an ongoing stream
    pub async fn connect_stream(
        &self,
        user_id: &Uuid,
        session_id: &Uuid,
    ) -> Result<StreamAccessResponse, ChatError> {
        let stream_key = StreamingService::chat_stream_key(user_id, session_id);
        let streams = StreamingService::new(self.tinistream);
        if !streams.exists_stream(&stream_key).await? {
            return Err(ChatError::StreamNotFound);
        }

        Ok(streams.access_stream(&stream_key).await?)
    }

    /// Cancel an ongoing stream
    pub async fn cancel_stream(
        &self,
        user_id: &Uuid,
        session_id: &Uuid,
    ) -> Result<StreamStatus, ChatError> {
        let stream_key = StreamingService::chat_stream_key(user_id, session_id);
        let streams = StreamingService::new(self.tinistream);
        if !streams.exists_stream(&stream_key).await? {
            return Err(ChatError::StreamNotFound);
        }

        Ok(streams.cancel_stream(&stream_key).await?)
    }

    /// Get the currently streaming session IDs for the given user
    pub async fn active_stream_sessions(&self, user_id: &Uuid) -> Result<Vec<Uuid>, ChatError> {
        let prefix = StreamingService::chat_stream_prefix(user_id);
        let session_ids = StreamingService::new(self.tinistream)
            .active_streams(&prefix)
            .await?
            .iter()
            .filter_map(|stream| StreamingService::session_id_from_stream_key(&stream.key, &prefix))
            .collect();

        Ok(session_ids)
    }

    /// Send a single prompt to the LLM provider and stream the response
    pub async fn prompt(
        &self,
        db: &mut DbService,
        user_id: Uuid,
        provider_id: i32,
        provider: Arc<dyn LlmProvider>,
        prompt: LlmUserMessage,
        options: LlmChatOptions,
    ) -> Result<StreamAccessResponse, ChatError> {
        let log_id = db
            .logs()
            .create()
            .kind(ChatRsLogKind::Prompt)
            .user_id(&user_id)
            .provider_id(provider_id)
            .model(&options.model)
            .build()
            .await?;

        let request = LlmChatRequest {
            messages: &[LlmMessage::User(prompt)],
            options: &options,
        };
        let (response_stream, response_meta) = match provider.stream_chat(request).await {
            Ok(response) => response,
            Err(err) => {
                db.logs()
                    .complete()
                    .id(log_id)
                    .status(ChatRsLogStatus::Error)
                    .error(&err.to_string())
                    .maybe_request_id(err.req_id())
                    .build()
                    .await?;
                return Err(ChatError::Request(err));
            }
        };

        // Spawn thread to process LLM streaming response
        let stream_key = StreamingService::prompt_key(&user_id);
        let (stream_access, ws_writer, ws_reader) = StreamingService::new(self.tinistream)
            .create_stream(&stream_key)
            .await?;
        let db_pool = self.db_pool.to_owned();
        let tinistream_client = self.tinistream.to_owned();
        tokio::spawn(async move {
            let output =
                StreamingService::process_stream(response_stream, ws_writer, ws_reader).await;
            if let Ok(mut db) = DbService::from_pool(&db_pool).await {
                let _ = db
                    .logs()
                    .complete()
                    .id(log_id)
                    .status(output.status())
                    .maybe_request_id(response_meta.request_id.as_deref())
                    .maybe_usage(output.usage.as_ref())
                    .maybe_error(output.errors.map(|e| e.join(", ")).as_deref())
                    .build()
                    .await;
            }
            let _ = StreamingService::new(&tinistream_client)
                .end_stream(&stream_key)
                .await;
        });

        // Return the URL and token for the user to access the client stream
        Ok(stream_access)
    }

    /// Stream response to a user message in a session
    pub async fn stream_user_chat(
        &self,
        db: &mut DbService,
        user_id: Uuid,
        session_id: Uuid,
        provider_id: i32,
        provider: Arc<dyn LlmProvider>,
        user_message: Option<LlmUserMessage>,
        chat_options: LlmChatOptions,
    ) -> Result<StreamAccessResponse, ChatError> {
        let (chat_session, mut messages) = db
            .chats()
            .find_session_with_messages(&user_id, &session_id)
            .await?;
        let chat_session = chat_session.ok_or(ChatError::SessionNotFound)?;

        if let Some(user_message) = user_message {
            if messages.is_empty() && chat_session.title == DEFAULT_SESSION_TITLE {
                titles::generate_title(
                    user_id,
                    session_id,
                    provider_id,
                    &provider,
                    &user_message.text,
                    &chat_options.model,
                    self.db_pool,
                );
            }
            let new_message = db
                .chats()
                .save_message(NewChatRsMessage {
                    content: &user_message.text,
                    session_id: &session_id,
                    role: ChatRsMessageRole::User,
                    meta: ChatRsMessageMeta::new_user(UserMeta::default()),
                })
                .await?;
            messages.push(new_message);
        }

        self.start_assistant_stream(
            db,
            provider,
            messages,
            ChatStreamParams {
                user_id,
                session_id,
                provider_id,
                chat_options,
                replace_message_id: None,
            },
        )
        .await
    }

    /// Regenerate the last assistant response in a chat session
    pub async fn regenerate_response(
        &self,
        db: &mut DbService,
        user_id: Uuid,
        session_id: Uuid,
        provider_id: i32,
        provider: Arc<dyn LlmProvider>,
        chat_options: LlmChatOptions,
    ) -> Result<StreamAccessResponse, ChatError> {
        let (chat_session, mut messages) = db
            .chats()
            .find_session_with_messages(&user_id, &session_id)
            .await?;
        chat_session.ok_or(ChatError::SessionNotFound)?;

        let last_message = messages.pop();
        if last_message.as_ref().is_none_or(|m| !m.role.is_assistant()) {
            return Err(ChatError::NoAssistantResponse);
        }

        self.start_assistant_stream(
            db,
            provider,
            messages,
            ChatStreamParams {
                user_id,
                session_id,
                provider_id,
                chat_options,
                replace_message_id: last_message.map(|m| m.id),
            },
        )
        .await
    }

    /// Start the LLM response stream and return the access URL & token
    async fn start_assistant_stream(
        &self,
        db: &mut DbService,
        provider: Arc<dyn LlmProvider>,
        messages: Vec<ChatRsMessage>,
        params: ChatStreamParams,
    ) -> Result<StreamAccessResponse, ChatError> {
        let streams = StreamingService::new(self.tinistream);
        let stream_key = StreamingService::chat_stream_key(&params.user_id, &params.session_id);
        if streams.exists_stream(&stream_key).await? {
            return Err(ChatError::AlreadyStreaming);
        }

        let log_id = db
            .logs()
            .create()
            .kind(ChatRsLogKind::Chat)
            .user_id(&params.user_id)
            .session_id(&params.session_id)
            .provider_id(params.provider_id)
            .model(&params.chat_options.model)
            .build()
            .await?;

        let (response_stream, meta) = match provider
            .stream_chat(LlmChatRequest {
                messages: &messages::build_llm_messages(messages)?,
                options: &params.chat_options,
            })
            .await
        {
            Ok(response) => response,
            Err(err) => {
                db.logs()
                    .complete()
                    .id(log_id)
                    .status(ChatRsLogStatus::Error)
                    .error(&err.to_string())
                    .maybe_request_id(err.req_id())
                    .build()
                    .await?;
                return Err(ChatError::Request(err));
            }
        };

        // Spawn thread to process and save LLM streaming response
        let (stream_access, ws_writer, ws_reader) = streams.create_stream(&stream_key).await?;
        let db_pool = self.db_pool.to_owned();
        let tinistream_client = self.tinistream.to_owned();
        tokio::spawn(async move {
            let output =
                StreamingService::process_stream(response_stream, ws_writer, ws_reader).await;
            let stream_cancelled = output.cancelled;
            if let Err(err) = Self::persist_response(output, params, log_id, meta, db_pool).await {
                tracing::error!("Failed to save assistant response: {err}");
            }

            if !stream_cancelled {
                let _ = StreamingService::new(&tinistream_client)
                    .end_stream(&stream_key)
                    .await;
            }
        });

        // Return the URL and token for the user to access the client stream
        Ok(stream_access)
    }

    /// Save response message and metadata to database
    async fn persist_response(
        output: LlmStreamOutput,
        params: ChatStreamParams,
        log_id: i32,
        meta: LlmResponseMeta,
        db_pool: DbPool,
    ) -> Result<ChatRsMessage, ChatError> {
        let completed_at = chrono::Utc::now();
        let mut db = DbService::from_pool(&db_pool).await?;

        let assistant_meta = AssistantMeta {
            provider_id: params.provider_id,
            provider_options: Some(params.chat_options),
            // tool_calls: response.tool_calls,
            // files: image_ids,
            usage: output.usage,
            errors: output.errors.clone(),
            partial: output.cancelled.then_some(true),
            ..Default::default()
        };
        let new_message = db
            .chats()
            .save_message(NewChatRsMessage {
                content: output.text.as_deref().unwrap_or_default(),
                meta: ChatRsMessageMeta::new_assistant(assistant_meta),
                role: ChatRsMessageRole::Assistant,
                session_id: &params.session_id,
            })
            .await?;
        if let Some(message_id) = params.replace_message_id {
            db.chats()
                .delete_message(&params.session_id, &message_id)
                .await?;
        }

        db.logs()
            .complete()
            .id(log_id)
            .status(output.status())
            .completed_at(completed_at)
            .message_id(&new_message.id)
            .maybe_request_id(meta.request_id.as_deref())
            .maybe_usage(output.usage.as_ref())
            .maybe_error(output.errors.map(|e| e.join(", ")).as_deref())
            .build()
            .await?;

        Ok(new_message)
    }
}
