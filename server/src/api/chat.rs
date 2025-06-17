use std::{borrow::Cow, pin::Pin};

use rocket::{
    futures::{Stream, StreamExt},
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
        services::{api_key::ApiKeyDbService, chat::ChatDbService},
        DbConnection, DbPool,
    },
    errors::ApiError,
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
    provider: ProviderConfigInput,
}

/// Send a chat message and stream the response
#[openapi(tag = "Chat")]
#[post("/<session_id>", data = "<input>")]
pub async fn send_chat_stream(
    user: ChatRsUser,
    db_pool: &State<DbPool>,
    mut db: DbConnection,
    redis: RedisClient,
    encryptor: &State<Encryptor>,
    session_id: Uuid,
    input: Json<SendChatInput<'_>>,
) -> Result<EventStream<Pin<Box<dyn Stream<Item = Event> + Send>>>, ApiError> {
    // Check session exists and user is owner, get message history
    let (_, current_messages) = ChatDbService::new(&mut db)
        .get_session_with_messages(&user.id, &session_id)
        .await?;
    let is_first_message = current_messages.is_empty();

    // Get the chat provider
    let provider = create_provider(
        &user.id,
        &input.provider,
        &mut ApiKeyDbService::new(&mut db),
        &encryptor,
    )
    .await?;

    // Get the provider's stream response and wrap it in our StoredChatRsStream
    let stream = StoredChatRsStream::new(
        provider
            .chat_stream(input.message.as_deref(), Some(current_messages))
            .await?,
        input.provider.clone(),
        db_pool.inner().clone(),
        redis.clone(),
        Some(session_id),
    );

    // Save user message to session, and generate title if needed
    if let Some(user_message) = &input.message {
        let _ = ChatDbService::new(&mut db)
            .save_message(NewChatRsMessage {
                content: user_message,
                session_id: &session_id,
                role: ChatRsMessageRole::User,
                meta: &ChatRsMessageMeta::default(),
            })
            .await?;
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

    // Start streaming
    let event_stream: Pin<Box<dyn Stream<Item = Event> + Send>> =
        Box::pin(stream.map(|result| match result {
            Ok(message) => Event::data(format!(" {message}")).event("chat"),
            Err(err) => Event::data(err.to_string()).event("error"),
        }));
    Ok(EventStream::from(event_stream))
}
