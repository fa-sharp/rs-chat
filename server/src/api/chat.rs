use std::{borrow::Cow, pin::Pin};

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
            ChatRsMessageMeta, ChatRsMessageRole, ChatRsSessionMeta, NewChatRsMessage,
            UpdateChatRsSession,
        },
        services::{ChatDbService, ProviderDbService, ToolDbService},
        DbConnection, DbPool,
    },
    errors::ApiError,
    provider::{build_llm_provider_api, LlmApiProviderSharedOptions, LlmError},
    stream::{LlmStreamWriter, SseStreamReader},
    tools::{get_llm_tools_from_input, SendChatToolInput},
    utils::{generate_title, Encryptor},
};

pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        settings: get_chat_streams,
        send_chat_stream,
        connect_to_chat_stream,
        cancel_chat_stream,
    ]
}

#[derive(Debug, JsonSchema, serde::Serialize)]
pub struct GetChatStreamsResponse {
    streams: Vec<String>,
}

/// # Get chat streams
/// Get the ongoing chat response streams
#[openapi(tag = "Chat")]
#[get("/streams")]
pub async fn get_chat_streams(
    user_id: ChatRsUserId,
    redis: &State<fred::prelude::Pool>,
) -> Result<Json<GetChatStreamsResponse>, ApiError> {
    let stream_reader = SseStreamReader::new(&redis);
    let keys = stream_reader.get_chat_streams(&user_id).await?;
    Ok(Json(GetChatStreamsResponse { streams: keys }))
}

#[derive(JsonSchema, serde::Deserialize)]
pub struct SendChatInput<'a> {
    /// The new chat message from the user
    message: Option<Cow<'a, str>>,
    /// The ID of the provider to chat with
    provider_id: i32,
    /// Configuration for the provider
    options: LlmApiProviderSharedOptions,
    /// Configuration of tools available to the assistant
    tools: Option<SendChatToolInput>,
}

/// # Start chat stream
/// Send a chat message and start the streamed assistant response. After the response
/// has started, use the `/<session_id>/stream` endpoint to connect to the SSE stream.
#[openapi(tag = "Chat")]
#[post("/<session_id>", data = "<input>")]
pub async fn send_chat_stream(
    user_id: ChatRsUserId,
    db_pool: &State<DbPool>,
    mut db: DbConnection,
    redis: &State<fred::prelude::Pool>,
    encryptor: &State<Encryptor>,
    http_client: &State<reqwest::Client>,
    session_id: Uuid,
    mut input: Json<SendChatInput<'_>>,
) -> Result<String, ApiError> {
    let mut stream_writer = LlmStreamWriter::new(&redis, &user_id, &session_id);

    // Check that we aren't already streaming a response for this session
    if stream_writer.exists().await? {
        return Err(LlmError::AlreadyStreaming)?;
    }

    // Get session and message history
    let (session, mut messages) = ChatDbService::new(&mut db)
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
        redis,
    )?;

    // Get the user's chosen tools
    let mut tools = None;
    if let Some(tool_input) = input.tools.as_ref() {
        let mut tool_db_service = ToolDbService::new(&mut db);
        tools = Some(get_llm_tools_from_input(&user_id, tool_input, &mut tool_db_service).await?);
    }

    // Generate session title if needed, and save user message to database
    if let Some(user_message) = &input.message {
        if messages.is_empty() && session.title == DEFAULT_SESSION_TITLE {
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
        messages.push(new_message);
    }

    // Update session metadata if needed
    if let Some(tool_input) = input.tools.take() {
        if session
            .meta
            .tool_config
            .is_none_or(|config| config != tool_input)
        {
            let meta = ChatRsSessionMeta::new(Some(tool_input));
            let data = UpdateChatRsSession {
                meta: Some(&meta),
                ..Default::default()
            };
            ChatDbService::new(&mut db)
                .update_session(&user_id, &session_id, data)
                .await?;
        }
    }

    // Get the provider's stream response
    let stream = provider_api
        .chat_stream(messages, tools, &input.options)
        .await?;
    let provider_id = input.provider_id;
    let provider_options = input.options.clone();

    // Create the Redis stream, then spawn a task to stream and save the response
    stream_writer.create().await?;
    tokio::spawn(async move {
        let mut chat_db_service = ChatDbService::new(&mut db);
        stream_writer
            .process(stream, &mut chat_db_service, provider_id, provider_options)
            .await;
    });

    Ok(format!("Stream started at /api/chat/{}/stream", session_id))
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
    let stream_reader = SseStreamReader::new(&redis);

    // Get all previous events from the Redis stream, and return them if we're already at the end of the stream
    let (prev_events, last_event_id, is_end) =
        stream_reader.get_prev_events(&user_id, &session_id).await?;
    let prev_events_stream = stream::iter(prev_events);
    if is_end {
        return Ok(EventStream::from(prev_events_stream.boxed()));
    }

    // Spawn a task to receive new events from Redis and add them to this channel
    let (tx, rx) = tokio::sync::mpsc::channel::<Event>(50);
    tokio::spawn(async move {
        stream_reader
            .stream(&user_id, &session_id, &last_event_id, &tx)
            .await;
        drop(tx);
    });

    // Send stream to client
    let stream = prev_events_stream.chain(ReceiverStream::new(rx)).boxed();
    Ok(EventStream::from(stream))
}

/// # Cancel chat stream
/// Cancel an ongoing chat stream
#[openapi(tag = "Chat")]
#[post("/<session_id>/cancel")]
pub async fn cancel_chat_stream(
    user_id: ChatRsUserId,
    redis: &State<fred::prelude::Pool>,
    session_id: Uuid,
) -> Result<(), ApiError> {
    let stream_writer = LlmStreamWriter::new(&redis, &user_id, &session_id);
    if !stream_writer.exists().await? {
        return Err(LlmError::StreamNotFound)?;
    }
    stream_writer.cancel().await?;
    Ok(())
}
