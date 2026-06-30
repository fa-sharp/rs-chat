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
        types::{LlmChatOptions, LlmChatRequest, LlmUserMessage},
    },
    services::{
        chat::error::ChatError,
        stream::{LlmStreamOutput, StreamingService, tinistream::TinistreamClient},
    },
};

mod error;
mod messages;

pub struct ChatService<'r> {
    db_pool: &'r DbPool,
    tinistream: &'r TinistreamClient,
}

impl<'r> ChatService<'r> {
    pub fn new(db_pool: &'r DbPool, tinistream: &'r TinistreamClient) -> Self {
        Self {
            db_pool,
            tinistream,
        }
    }

    pub async fn stream_user_chat(
        &self,
        user_id: Uuid,
        session_id: Uuid,
        provider: &dyn LlmProvider,
        provider_id: i32,
        user_message: LlmUserMessage,
        chat_options: LlmChatOptions,
    ) -> Result<StreamAccessResponse, ChatError> {
        // Check for existing chat session, then save the new user message to it
        let mut db = DbService::from_pool(self.db_pool).await?;
        let (_existing_session, mut session_messages) = db
            .chats()
            .get_session_with_messages(&user_id, &session_id)
            .await?;
        let new_message = db
            .chats()
            .save_message(NewChatRsMessage {
                session_id: &session_id,
                role: ChatRsMessageRole::User,
                content: &user_message.text,
                meta: ChatRsMessageMeta::new_user(UserMeta::default()),
            })
            .await?;
        session_messages.push(new_message);

        // Send the request to the LLM provider and get the streaming response
        let llm_messages = messages::build_llm_messages(session_messages)?;
        let stream = provider
            .stream_chat(LlmChatRequest {
                messages: &llm_messages,
                options: &chat_options,
            })
            .await?;

        // Create a new client stream in `tinistream` to stream the response to the user
        let stream_key = StreamingService::chat_stream_key(&user_id, &session_id);
        let (stream_access, ws_writer, ws_reader) = StreamingService::new(self.tinistream)
            .create_stream(&stream_key)
            .await?;

        // Spawn task to process and save the streaming response
        let db_pool = self.db_pool.to_owned();
        let tinistream_client = self.tinistream.to_owned();
        tokio::spawn(async move {
            let response = StreamingService::process_stream(stream, ws_writer, ws_reader).await;
            let stream_cancelled = response.cancelled;
            if let Err(err) =
                Self::persist_response(db_pool, &session_id, provider_id, chat_options, response)
                    .await
            {
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
        db_pool: DbPool,
        session_id: &Uuid,
        provider_id: i32,
        chat_options: LlmChatOptions,
        response: LlmStreamOutput,
    ) -> Result<ChatRsMessage, ChatError> {
        let mut db = DbService::from_pool(&db_pool).await?;
        let assistant_meta = AssistantMeta {
            provider_id,
            provider_options: Some(chat_options),
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
                session_id: &session_id,
            })
            .await?;

        Ok(new_message)
    }
}
