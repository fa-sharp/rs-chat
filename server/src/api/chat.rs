use std::borrow::Cow;

use rocket::{get, post, serde::json::Json, Route, State};
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
};
use schemars::JsonSchema;
use uuid::Uuid;

use crate::{
    api::session::DEFAULT_SESSION_TITLE,
    auth::ChatRsUserId,
    db::{
        models::*,
        services::{ChatDbService, ProviderDbService, ToolDbService},
        DbConnection, DbPool,
    },
    errors::ApiError,
    provider::{build_llm_messages, build_llm_provider_api, LlmError, LlmProviderOptions},
    redis::RedisClient,
    storage::LocalStorage,
    stream::*,
    tools::SendChatToolInput,
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
    sessions: Vec<String>,
}

/// # Get chat streams
/// Get the session IDs that have ongoing chat response streams
#[openapi(tag = "Chat")]
#[get("/streams")]
pub async fn get_chat_streams(
    user_id: ChatRsUserId,
    tinistream: &State<TinistreamClient>,
) -> Result<Json<GetChatStreamsResponse>, ApiError> {
    let prefix = chat_stream_prefix(&user_id);
    let sessions = tinistream
        .active_streams(&prefix)
        .await?
        .iter()
        .filter_map(|stream| stream.key.strip_prefix(&prefix).map(String::from))
        .collect();
    Ok(Json(GetChatStreamsResponse { sessions }))
}

#[derive(JsonSchema, serde::Deserialize)]
pub struct SendChatInput<'a> {
    /// The new chat message from the user
    message: Option<Cow<'a, str>>,
    /// The ID of the provider to chat with
    provider_id: i32,
    /// Configuration for the provider
    options: LlmProviderOptions,
    /// Configuration of tools available to the assistant
    tools: Option<SendChatToolInput>,
    /// IDs of the file(s) to attach to this message
    files: Option<Vec<Uuid>>,
}

#[derive(JsonSchema, serde::Serialize)]
pub struct SendChatResponse {
    message: &'static str,
    url: String,
    token: String,
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
    redis: RedisClient,
    tinistream: &State<TinistreamClient>,
    encryptor: &State<Encryptor>,
    storage: &State<LocalStorage>,
    http_client: &State<reqwest::Client>,
    session_id: Uuid,
    mut input: Json<SendChatInput<'_>>,
) -> Result<Json<SendChatResponse>, ApiError> {
    // Check that we aren't already streaming a response for this session
    let stream_key = chat_stream_key(&user_id, &session_id);
    if tinistream.stream_exists(&stream_key).await? {
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
        &redis,
    )?;

    // Get the user's chosen tools
    let mut tools = None;
    if let Some(conf) = input.tools.take() {
        let llm_tools = conf
            .get_llm_tools(&user_id, &mut ToolDbService::new(&mut db))
            .await?;
        tools = Some(llm_tools);

        // Update session metadata with new tools if needed
        if session.meta.tool_config.as_ref().is_none_or(|c| *c != conf) {
            let data = UpdateChatRsSession {
                meta: Some(ChatRsSessionMeta::new(Some(conf))),
                ..Default::default()
            };
            ChatDbService::new(&mut db)
                .update_session(&user_id, &session_id, data)
                .await?;
        }
    }

    // Generate session title if needed, and save user message to database
    let attached_file_ids = input.files.take();
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
        let message_meta = attached_file_ids
            .map(|ids| ChatRsMessageMeta::new_user(UserMeta { files: Some(ids) }))
            .unwrap_or_default();
        let message = ChatDbService::new(&mut db)
            .save_message(NewChatRsMessage {
                content: user_message,
                session_id: &session_id,
                role: ChatRsMessageRole::User,
                meta: message_meta,
            })
            .await?;
        messages.push(message);
    }

    // Convert the messages, and get the provider's response stream
    let llm_messages =
        build_llm_messages(messages, &user_id, &session_id, &mut db, &storage).await?;
    let stream = provider_api
        .chat_stream(llm_messages, tools, &input.options)
        .await?;

    // Create the Redis stream
    let mut stream_writer = LlmStreamWriter::new(tinistream.inner().clone(), &user_id, &session_id);
    let stream_access = stream_writer.start().await?;

    // Spawn a task to stream and save the response
    let provider_id = input.provider_id.clone();
    let provider_options = input.options.clone();
    tokio::spawn(async move {
        let (text, tool_calls, usage, errors, cancelled) = stream_writer.process(stream).await;
        let assistant_meta = AssistantMeta {
            provider_id,
            provider_options: Some(provider_options),
            tool_calls,
            usage,
            errors,
            partial: cancelled.then_some(true),
        };
        let db_result = ChatDbService::new(&mut db)
            .save_message(NewChatRsMessage {
                session_id: &session_id,
                role: ChatRsMessageRole::Assistant,
                content: &text.unwrap_or_default(),
                meta: ChatRsMessageMeta::new_assistant(assistant_meta),
            })
            .await;
        if let Err(err) = db_result {
            rocket::error!("Failed to save assistant message: {}", err);
        }
        if !cancelled {
            stream_writer.end().await.ok();
        }
    });

    Ok(Json(SendChatResponse {
        message: "Stream started",
        url: stream_access.url,
        token: stream_access.token,
    }))
}

#[derive(Debug, JsonSchema, serde::Serialize)]
pub struct StreamAccess {
    url: String,
    token: String,
}

/// # Connect to chat stream
/// Get a stream URL and token to stream the assistant response
#[openapi(tag = "Chat")]
#[get("/<session_id>/stream")]
pub async fn connect_to_chat_stream(
    user_id: ChatRsUserId,
    session_id: Uuid,
    tinistream: &State<TinistreamClient>,
) -> Result<Json<StreamAccess>, ApiError> {
    let key = chat_stream_key(&user_id, &session_id);
    let connect = tinistream.stream_connect(&key).await?;

    Ok(Json(StreamAccess {
        url: connect.url,
        token: connect.token,
    }))
}

/// # Cancel chat stream
/// Cancel an ongoing chat stream
#[openapi(tag = "Chat")]
#[post("/<session_id>/cancel")]
pub async fn cancel_chat_stream(
    user_id: ChatRsUserId,
    session_id: Uuid,
    tinistream: &State<TinistreamClient>,
) -> Result<(), ApiError> {
    let key = chat_stream_key(&user_id, &session_id);
    tinistream.stream_cancel(&key).await?;

    Ok(())
}
