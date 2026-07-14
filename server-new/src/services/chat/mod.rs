use std::sync::Arc;

use tinistream_client::types::StreamAccessResponse;
use uuid::Uuid;

use crate::{
    db::{
        DbPool, DbService,
        models::{
            AssistantMeta, ChatRsMessage, ChatRsMessageMeta, ChatRsMessageRole, NewChatRsMessage,
            UserMeta,
        },
    },
    llm::{
        interface::LlmProvider,
        types::{LlmChatOptions, LlmChatRequest, LlmPrompt, LlmUserMessage},
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

/// Chat stream parameters for response generation
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

    /// Send a simple prompt to the LLM provider
    pub async fn prompt(
        &self,
        provider: Arc<dyn LlmProvider>,
        prompt: LlmUserMessage,
        options: LlmChatOptions,
    ) -> Result<String, ChatError> {
        let llm_prompt = LlmPrompt {
            text: &prompt.text,
            options: &options,
        };
        Ok(provider.prompt(llm_prompt).await?)
    }

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
                    &user_message.text,
                    &provider,
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

    pub async fn regenerate_response(
        &self,
        db: &mut DbService,
        user_id: Uuid,
        session_id: Uuid,
        provider_id: i32,
        provider: Arc<dyn LlmProvider>,
        chat_options: LlmChatOptions,
    ) -> Result<StreamAccessResponse, ChatError> {
        let (chat_session, messages) = db
            .chats()
            .find_session_with_messages(&user_id, &session_id)
            .await?;
        chat_session.ok_or(ChatError::SessionNotFound)?;
        let assistant_message_id = messages
            .iter()
            .rev()
            .find(|m| m.role.is_assistant())
            .ok_or(ChatError::NoAssistantResponse)?
            .id;

        self.start_assistant_stream(
            provider,
            messages,
            ChatStreamParams {
                user_id,
                session_id,
                provider_id,
                chat_options,
                replace_message_id: Some(assistant_message_id),
            },
        )
        .await
    }

    async fn start_assistant_stream(
        &self,
        provider: Arc<dyn LlmProvider>,
        messages: Vec<ChatRsMessage>,
        params: ChatStreamParams,
    ) -> Result<StreamAccessResponse, ChatError> {
        let stream_key = StreamingService::chat_stream_key(&params.user_id, &params.session_id);
        let stream_service = StreamingService::new(self.tinistream);
        if stream_service.exists_stream(&stream_key).await? {
            return Err(ChatError::AlreadyStreaming);
        }

        let llm_messages = messages::build_llm_messages(messages)?;
        let response_stream = provider
            .stream_chat(LlmChatRequest {
                messages: &llm_messages,
                options: &params.chat_options,
            })
            .await?;

        let (stream_access, ws_writer, ws_reader) =
            stream_service.create_stream(&stream_key).await?;

        let db_pool = self.db_pool.to_owned();
        let tinistream_client = self.tinistream.to_owned();
        tokio::spawn(async move {
            let response =
                StreamingService::process_stream(response_stream, ws_writer, ws_reader).await;
            let stream_cancelled = response.cancelled;
            if let Err(err) = Self::persist_response(response, params, db_pool).await {
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
        response: LlmStreamOutput,
        params: ChatStreamParams,
        db_pool: DbPool,
    ) -> Result<ChatRsMessage, ChatError> {
        let mut db = DbService::from_pool(&db_pool).await?;
        let assistant_meta = AssistantMeta {
            provider_id: params.provider_id,
            provider_options: Some(params.chat_options),
            // tool_calls: response.tool_calls,
            // files: image_ids,
            usage: response.usage,
            errors: response.errors,
            partial: response.cancelled.then_some(true),
            ..Default::default()
        };
        let new_message = db
            .chats()
            .save_message(NewChatRsMessage {
                content: &response.text.unwrap_or_default(),
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

        Ok(new_message)
    }
}
