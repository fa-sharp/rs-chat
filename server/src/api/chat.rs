use std::{borrow::Cow, pin::Pin};

use rocket::{
    futures::{try_join, Stream, StreamExt},
    post,
    response::stream::{Event, EventStream},
    serde::json::Json,
    Route, State,
};
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
};
use schemars::JsonSchema;
use uuid::Uuid;

use crate::{
    db::{
        models::{ChatRsMessageMeta, ChatRsMessageRole, ChatRsUser, NewChatRsMessage},
        services::{api_key::ApiKeyDbService, chat::ChatDbService, storage::StorageDbService},
        DbConnection, DbPool,
    },
    errors::ApiError,
    provider::ChatRsProviderMessage,
    redis::RedisClient,
    utils::{
        create_provider::{create_provider, ProviderConfigInput},
        encryption::Encryptor,
        generate_title::generate_title,
        stored_stream::StoredChatRsStream,
    },
};

pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: send_chat_stream]
}

#[derive(JsonSchema, serde::Deserialize)]
pub struct SendChatInput<'a> {
    message: Option<Cow<'a, str>>,
    attachments: Option<Vec<Uuid>>,
    provider: ProviderConfigInput,
}

/// Send a chat message and stream the response
#[openapi(tag = "Chat")]
#[post("/<session_id>", data = "<input>")]
pub async fn send_chat_stream(
    user: ChatRsUser,
    db_pool: &State<DbPool>,
    mut db_1: DbConnection,
    mut db_2: DbConnection,
    redis: RedisClient,
    encryptor: &State<Encryptor>,
    session_id: Uuid,
    input: Json<SendChatInput<'_>>,
) -> Result<EventStream<Pin<Box<dyn Stream<Item = Event> + Send>>>, ApiError> {
    // Get current session with message and attachment history
    let mut chat_db_service = ChatDbService::new(&mut db_1);
    let mut storage_db_service = StorageDbService::new(&mut db_2);
    let ((_, current_messages), current_attachments) = try_join!(
        chat_db_service.get_session_with_messages(&user.id, &session_id),
        storage_db_service.find_by_session(&session_id)
    )?;
    let is_first_message = current_messages.is_empty();

    let mut messages: Vec<ChatRsProviderMessage> = current_messages
        .into_iter()
        .map(|msg| ChatRsProviderMessage::Message(msg))
        .collect();
    for (message_id, file) in current_attachments {
        if let Some(message_idx) = messages.iter().position(|msg| match msg {
            ChatRsProviderMessage::Message(message) => message.id == message_id,
            _ => false,
        }) {
            messages.insert(message_idx + 1, ChatRsProviderMessage::Attachment(file));
        }
    }

    // Save user message and attachments, and generate title if needed
    if let Some(user_message) = &input.message {
        let new_message = chat_db_service
            .save_message(NewChatRsMessage {
                content: user_message,
                session_id: &session_id,
                role: ChatRsMessageRole::User,
                meta: &ChatRsMessageMeta::default(),
            })
            .await?;
        let new_attachments = match input.attachments.as_ref() {
            Some(file_ids) => {
                StorageDbService::new(&mut db_2)
                    .attach_files_to_message(&user.id, &new_message.id, file_ids)
                    .await?
            }
            None => Vec::new(),
        };
        messages.push(ChatRsProviderMessage::Message(new_message));
        messages.extend(
            new_attachments
                .into_iter()
                .map(|file| ChatRsProviderMessage::Attachment(file)),
        );

        if is_first_message {
            generate_title(
                &user.id,
                &session_id,
                user_message,
                &input.provider,
                &encryptor,
                db_pool,
            );
        }
    }

    // Get the chat provider
    let provider = create_provider(
        &user.id,
        &input.provider,
        &mut ApiKeyDbService::new(&mut db_2),
        &encryptor,
    )
    .await?;

    // Get the provider's stream response and wrap it in our StoredChatRsStream
    let stream = StoredChatRsStream::new(
        provider.chat_stream(messages).await?,
        input.provider.clone(),
        db_pool.inner().clone(),
        redis.clone(),
        Some(session_id),
    );

    // Start streaming
    let event_stream: Pin<Box<dyn Stream<Item = Event> + Send>> =
        Box::pin(stream.map(|result| match result {
            Ok(message) => Event::data(format!(" {message}")).event("chat"),
            Err(err) => Event::data(err.to_string()).event("error"),
        }));
    Ok(EventStream::from(event_stream))
}
