use std::{borrow::Cow, pin::Pin};

use fred::prelude::KeysInterface;
use rocket::{
    futures::{stream, Stream, StreamExt},
    get, post,
    response::stream::{Event, EventStream},
    serde::json::Json,
    Route, State,
};
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
};
use schemars::JsonSchema;
use tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;

use crate::{
    api::session::DEFAULT_SESSION_TITLE,
    auth::ChatRsUserId,
    db::{
        models::{
            AssistantMeta, ChatRsMessageMeta, ChatRsMessageRole, ChatRsSessionMeta,
            NewChatRsMessage, UpdateChatRsSession,
        },
        services::{ChatDbService, ProviderDbService, ToolDbService},
        DbConnection, DbPool,
    },
    errors::ApiError,
    provider::{build_llm_provider_api, LlmApiProviderSharedOptions, LlmError},
    redis::RedisClient,
    stream::{LlmStreamWriter, SseStreamReader},
    tools::{get_llm_tools_from_input, SendChatToolInput},
    utils::{generate_title, Encryptor, StoredChatRsStream},
};

pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: send_chat_stream, send_chat_stream_v2, connect_to_chat_stream]
}

#[derive(JsonSchema, serde::Deserialize)]
pub struct SendChatInput<'a> {
    /// The new chat message from the user
    message: Option<Cow<'a, str>>,
    /// The ID of the provider to chat with
    provider_id: i32,
    /// Provider options
    provider_options: LlmApiProviderSharedOptions,
    /// Configuration of tools available to the assistant
    tools: Option<SendChatToolInput>,
}

/// Send a chat message and stream the response
#[openapi(tag = "Chat")]
#[post("/<session_id>/v1", data = "<input>")]
pub async fn send_chat_stream(
    user_id: ChatRsUserId,
    db_pool: &State<DbPool>,
    mut db: DbConnection,
    redis: RedisClient,
    encryptor: &State<Encryptor>,
    http_client: &State<reqwest::Client>,
    session_id: Uuid,
    mut input: Json<SendChatInput<'_>>,
) -> Result<EventStream<Pin<Box<dyn Stream<Item = Event> + Send>>>, ApiError> {
    // Check session exists and user is owner, get message history
    let (session, mut current_messages) = ChatDbService::new(&mut db)
        .get_session_with_messages(&user_id, &session_id)
        .await?;

    // Build the chat provider
    let (provider, api_key_secret) = ProviderDbService::new(&mut db)
        .get_by_id(&user_id, input.provider_id)
        .await?;
    let api_key = api_key_secret
        .map(|secret| encryptor.decrypt_string(&secret.ciphertext, &secret.nonce))
        .transpose()?;
    let provider_api = build_llm_provider_api(
        &provider.provider_type.as_str().try_into()?,
        provider.base_url.as_deref(),
        api_key.as_deref(),
        &http_client,
        &redis,
    )?;

    // Get the user's chosen tools
    let llm_tools = match input.tools.as_ref() {
        Some(tool_input) => {
            let mut tool_db_service = ToolDbService::new(&mut db);
            Some(get_llm_tools_from_input(&user_id, tool_input, &mut tool_db_service).await?)
        }
        None => None,
    };

    // Save user message and generate session title if needed
    if let Some(user_message) = &input.message {
        if current_messages.is_empty() && session.title == DEFAULT_SESSION_TITLE {
            generate_title(
                &user_id,
                &session_id,
                &user_message,
                &provider_api,
                &provider.default_model,
                db_pool,
            );
        }
        let new_message = ChatDbService::new(&mut db)
            .save_message(NewChatRsMessage {
                content: user_message,
                session_id: &session_id,
                role: ChatRsMessageRole::User,
                meta: ChatRsMessageMeta::default(),
            })
            .await?;
        current_messages.push(new_message);
    }

    // Update session metadata
    if let Some(tool_input) = input.tools.take() {
        if session
            .meta
            .tool_config
            .is_none_or(|config| config != tool_input)
        {
            ChatDbService::new(&mut db)
                .update_session(
                    &user_id,
                    &session_id,
                    UpdateChatRsSession {
                        meta: Some(&ChatRsSessionMeta {
                            tool_config: Some(tool_input),
                        }),
                        ..Default::default()
                    },
                )
                .await?;
        }
    }

    // Get the provider's stream response and wrap it in our StoredChatRsStream
    let stream = StoredChatRsStream::new(
        provider_api
            .chat_stream(current_messages, llm_tools, &input.provider_options)
            .await?,
        input.provider_id,
        input.provider_options.clone(),
        db_pool.inner().clone(),
        redis.clone(),
        Some(session_id),
    );

    // Start streaming
    let event_stream = stream
        .map(|result| match result {
            Ok(message) => Event::data(format!(" {message}")).event("chat"),
            Err(err) => Event::data(err.to_string()).event("error"),
        })
        .boxed();
    Ok(EventStream::from(event_stream))
}

/// # Start chat stream
/// Send a chat message and start the streamed assistant response. After the response
/// has started, use the `/<session_id>/stream` endpoint to connect to the SSE stream.
#[openapi(tag = "Chat")]
#[post("/<session_id>", data = "<input>")]
pub async fn send_chat_stream_v2(
    user_id: ChatRsUserId,
    db_pool: &State<DbPool>,
    mut db: DbConnection,
    redis: &State<fred::prelude::Pool>,
    encryptor: &State<Encryptor>,
    http_client: &State<reqwest::Client>,
    session_id: Uuid,
    mut input: Json<SendChatInput<'_>>,
) -> Result<String, ApiError> {
    // Check that we aren't already streaming a response for this session
    let stream_key = format!("user:{}:chat:{}", user_id.0, session_id);
    if redis.exists(&stream_key).await? {
        return Err(LlmError::AlreadyStreaming)?;
    }

    // Check session exists and user is owner, get message history
    let (session, mut current_messages) = ChatDbService::new(&mut db)
        .get_session_with_messages(&user_id, &session_id)
        .await?;

    // Build the LLM provider
    let (provider, api_key_secret) = ProviderDbService::new(&mut db)
        .get_by_id(&user_id, input.provider_id)
        .await?;
    let api_key = api_key_secret
        .map(|secret| encryptor.decrypt_string(&secret.ciphertext, &secret.nonce))
        .transpose()?;
    let provider_api = build_llm_provider_api(
        &provider.provider_type.as_str().try_into()?,
        provider.base_url.as_deref(),
        api_key.as_deref(),
        &http_client,
        redis.next(),
    )?;

    // Get the user's chosen tools
    let mut llm_tools = None;
    if let Some(tool_input) = input.tools.as_ref() {
        let mut tool_db_service = ToolDbService::new(&mut db);
        let tools = get_llm_tools_from_input(&user_id, tool_input, &mut tool_db_service).await?;
        llm_tools = Some(tools);
    }

    // Generate session title if needed, and save user message to database
    if let Some(user_message) = &input.message {
        if current_messages.is_empty() && session.title == DEFAULT_SESSION_TITLE {
            generate_title(
                &user_id,
                &session_id,
                &user_message,
                &provider_api,
                &provider.default_model,
                db_pool,
            );
        }
        let new_message = ChatDbService::new(&mut db)
            .save_message(NewChatRsMessage {
                content: user_message,
                session_id: &session_id,
                role: ChatRsMessageRole::User,
                meta: ChatRsMessageMeta::default(),
            })
            .await?;
        current_messages.push(new_message);
    }

    // Update session metadata if needed
    if let Some(tool_input) = input.tools.take() {
        if session
            .meta
            .tool_config
            .is_none_or(|config| config != tool_input)
        {
            ChatDbService::new(&mut db)
                .update_session(
                    &user_id,
                    &session_id,
                    UpdateChatRsSession {
                        meta: Some(&ChatRsSessionMeta {
                            tool_config: Some(tool_input),
                        }),
                        ..Default::default()
                    },
                )
                .await?;
        }
    }

    // Get the provider's stream response
    let stream = provider_api
        .chat_stream(current_messages, llm_tools, &input.provider_options)
        .await?;
    let provider_id = input.provider_id;
    let provider_options = input.provider_options.clone();

    // Spawn a task to stream the response to Redis and save it to the database on completion
    let stream_writer = LlmStreamWriter::new(&redis);
    tokio::spawn(async move {
        let (content, tool_calls, usage, _) =
            stream_writer.process_stream(&stream_key, stream).await;
        if let Err(e) = ChatDbService::new(&mut db)
            .save_message(NewChatRsMessage {
                session_id: &session_id,
                role: ChatRsMessageRole::Assistant,
                content: &content.unwrap_or_default(),
                meta: ChatRsMessageMeta {
                    assistant: Some(AssistantMeta {
                        provider_id,
                        provider_options: Some(provider_options),
                        tool_calls,
                        usage,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            })
            .await
        {
            rocket::warn!("Failed to save assistant response: {}", e);
        }

        // TODO delete stream in Redis
    });

    Ok("Stream started".into())
}

/// # Connect to chat stream
/// Connect to an ongoing chat stream and stream the assistant response
#[openapi(tag = "Chat")]
#[get("/<session_id>/stream")]
pub async fn connect_to_chat_stream(
    user_id: ChatRsUserId,
    redis: &State<fred::prelude::Pool>,
    session_id: Uuid,
) -> Result<EventStream<Pin<Box<dyn Stream<Item = Event> + Send>>>, ApiError> {
    let stream_key = format!("user:{}:chat:{}", user_id.0, session_id);
    let stream_reader = SseStreamReader::new(&redis);

    // Get all previous events from the Redis stream, and return them if we're already at the end of the stream
    let (prev_events, last_event_id, is_end) = stream_reader.get_prev_events(&stream_key).await?;
    let prev_events_stream = stream::iter(prev_events);
    if is_end {
        return Ok(EventStream::from(prev_events_stream.boxed()));
    }

    // Spawn a task to receive new events from Redis and add them to the channel
    let (tx, rx) = tokio::sync::mpsc::channel::<Event>(50);
    tokio::spawn(async move {
        stream_reader
            .stream_events(&stream_key, &last_event_id, &tx)
            .await;
        drop(tx);
    });

    // Send stream to client
    let stream = prev_events_stream.chain(ReceiverStream::new(rx)).boxed();
    Ok(EventStream::from(stream))
}
