use std::{borrow::Cow, collections::HashMap, pin::Pin};

use fred::prelude::{KeysInterface, StreamsInterface};
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
use tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;

use crate::{
    api::session::DEFAULT_SESSION_TITLE,
    auth::ChatRsUserId,
    db::{
        models::{
            AssistantMeta, ChatRsMessageMeta, ChatRsMessageRole, ChatRsProviderType,
            ChatRsSessionMeta, NewChatRsMessage, UpdateChatRsSession,
        },
        services::{ChatDbService, ProviderDbService, ToolDbService},
        DbConnection, DbPool,
    },
    errors::ApiError,
    provider::{build_llm_provider_api, LlmApiProviderSharedOptions, LlmError, LlmTool},
    redis::RedisClient,
    tools::SendChatToolInput,
    utils::{generate_title, Encryptor, LlmStreamProcessor, StoredChatRsStream},
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
#[post("/<session_id>", data = "<input>")]
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
    let provider_type: ChatRsProviderType = provider.provider_type.as_str().try_into()?;
    let api_key = api_key_secret
        .map(|secret| encryptor.decrypt_string(&secret.ciphertext, &secret.nonce))
        .transpose()?;
    let provider_api = build_llm_provider_api(
        &provider_type,
        provider.base_url.as_deref(),
        api_key.as_deref(),
        &http_client,
        &redis,
    )?;

    // Get the user's chosen tools
    let mut llm_tools: Option<Vec<LlmTool>> = None;
    let mut tool_db_service = ToolDbService::new(&mut db);
    if let Some(system_tool_input) = input.tools.as_ref().and_then(|t| t.system.as_ref()) {
        let system_tools = tool_db_service.find_system_tools_by_user(&user_id).await?;
        let system_llm_tools = system_tool_input.get_llm_tools(&system_tools)?;
        llm_tools.get_or_insert_default().extend(system_llm_tools);
    }
    if let Some(external_apis_input) = input.tools.as_ref().and_then(|t| t.external_apis.as_ref()) {
        let external_api_tools = tool_db_service
            .find_external_api_tools_by_user(&user_id)
            .await?;
        for tool_input in external_apis_input {
            let api_llm_tools = tool_input.into_llm_tools(&external_api_tools)?;
            llm_tools.get_or_insert_default().extend(api_llm_tools);
        }
    }

    // Save user message and generate session title if needed
    if let Some(user_message) = &input.message {
        if current_messages.is_empty() && session.title == DEFAULT_SESSION_TITLE {
            generate_title(
                &user_id,
                &session_id,
                &user_message,
                provider_type,
                &provider.default_model,
                provider.base_url.as_deref(),
                api_key.clone(),
                &http_client,
                &redis,
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
/// Send a chat message and start streaming the response
#[openapi(tag = "Chat")]
#[post("/<session_id>/v2", data = "<input>")]
pub async fn send_chat_stream_v2(
    user_id: ChatRsUserId,
    db_pool: &State<DbPool>,
    mut db: DbConnection,
    redis: RedisClient,
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

    // Build the chat provider
    let (provider, api_key_secret) = ProviderDbService::new(&mut db)
        .get_by_id(&user_id, input.provider_id)
        .await?;
    let provider_type: ChatRsProviderType = provider.provider_type.as_str().try_into()?;
    let api_key = api_key_secret
        .map(|secret| encryptor.decrypt_string(&secret.ciphertext, &secret.nonce))
        .transpose()?;
    let provider_api = build_llm_provider_api(
        &provider_type,
        provider.base_url.as_deref(),
        api_key.as_deref(),
        &http_client,
        &redis,
    )?;

    // Get the user's chosen tools
    let mut llm_tools: Option<Vec<LlmTool>> = None;
    let mut tool_db_service = ToolDbService::new(&mut db);
    if let Some(system_tool_input) = input.tools.as_ref().and_then(|t| t.system.as_ref()) {
        let system_tools = tool_db_service.find_system_tools_by_user(&user_id).await?;
        let system_llm_tools = system_tool_input.get_llm_tools(&system_tools)?;
        llm_tools.get_or_insert_default().extend(system_llm_tools);
    }
    if let Some(external_apis_input) = input.tools.as_ref().and_then(|t| t.external_apis.as_ref()) {
        let external_api_tools = tool_db_service
            .find_external_api_tools_by_user(&user_id)
            .await?;
        for tool_input in external_apis_input {
            let api_llm_tools = tool_input.into_llm_tools(&external_api_tools)?;
            llm_tools.get_or_insert_default().extend(api_llm_tools);
        }
    }

    // Save user message and generate session title if needed
    if let Some(user_message) = &input.message {
        if current_messages.is_empty() && session.title == DEFAULT_SESSION_TITLE {
            generate_title(
                &user_id,
                &session_id,
                &user_message,
                provider_type,
                &provider.default_model,
                provider.base_url.as_deref(),
                api_key.clone(),
                &http_client,
                &redis,
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

    // Get the provider's stream response, and spawn a task to stream it to Redis
    // and save the response to the database on completion
    let stream = provider_api
        .chat_stream(current_messages, llm_tools, &input.provider_options)
        .await?;
    let stream_processor = LlmStreamProcessor::new(&redis);
    let provider_id = input.provider_id;
    let provider_options = input.provider_options.clone();
    tokio::spawn(async move {
        let (content, tool_calls, usage, _) = stream_processor
            .process_llm_stream(&stream_key, stream)
            .await;
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
#[post("/<session_id>/stream")]
pub async fn connect_to_chat_stream(
    user_id: ChatRsUserId,
    redis: RedisClient,
    session_id: Uuid,
) -> Result<EventStream<Pin<Box<dyn Stream<Item = Event> + Send>>>, ApiError> {
    let stream_key = format!("user:{}:chat:{}", user_id.0, session_id);

    // Get all previous events from the Redis stream
    let (_, prev_values): (String, Vec<(String, HashMap<String, String>)>) = redis
        .xread::<Option<Vec<_>>, _, _>(None, None, &stream_key, "0-0")
        .await?
        .ok_or(LlmError::StreamNotFound)?
        .pop()
        .ok_or(LlmError::StreamNotFound)?;
    let last_event = prev_values.last().cloned();
    let prev_events_sse = prev_values
        .into_iter()
        .filter_map(convert_redis_event_to_sse);
    let prev_events_stream = rocket::futures::stream::iter(prev_events_sse);

    // If `end` event already received, just return previous events
    if let Some((_, ref data)) = last_event {
        if data.get("type").is_some_and(|t| t == "end") {
            return Ok(EventStream::from(prev_events_stream.boxed()));
        }
    }

    // Spawn a task to receive new events from Redis and add them to the channel
    let (tx, rx) = tokio::sync::mpsc::channel::<Event>(50);
    tokio::spawn(async move {
        let mut last_event_id = last_event.map(|(id, _)| id).unwrap_or_else(|| "0-0".into());
        loop {
            match get_next_event(&redis, &stream_key, &last_event_id, &tx).await {
                Ok(Some((id, event))) => {
                    last_event_id = id;
                    if let Err(_) = tx.send(event).await {
                        break; // client disconnected, stop sending events
                    }
                }
                Ok(None) => {
                    tx.send(Event::empty().event("end")).await.ok();
                    break; // No more events, end of stream
                }
                Err(e) => {
                    let event = Event::data(format!("Error: {}", e)).event("error");
                    tx.send(event).await.ok();
                    break;
                }
            }
        }
        drop(tx);
    });

    // Send stream of events from Redis to the client, starting with all previous events
    let stream = prev_events_stream.chain(ReceiverStream::new(rx)).boxed();
    Ok(EventStream::from(stream))
}

async fn get_next_event(
    redis: &RedisClient,
    stream_key: &str,
    last_event_id: &str,
    tx: &tokio::sync::mpsc::Sender<Event>,
) -> Result<Option<(String, Event)>, LlmError> {
    let next_stream_value: Option<Vec<(String, Vec<(String, HashMap<String, String>)>)>> = tokio::select! {
        next_value = redis.xread::<_, _, _>(Some(1), Some(5_000), stream_key, last_event_id) => next_value?,
        _ = tx.closed() => {
            println!("Client disconnected");
            return Ok(None);
        },
    };
    match next_stream_value
        .and_then(|mut v| v.pop())
        .and_then(|(_, mut events)| events.pop())
    {
        Some((id, event)) => {
            if event.get("type").is_some_and(|t| t == "end") {
                return Ok(None); // handle `end` event
            }
            Ok(Some(id.clone()).zip(convert_redis_event_to_sse((id, event))))
        }
        None => Ok(None),
    }
}

fn convert_redis_event_to_sse((id, event): (String, HashMap<String, String>)) -> Option<Event> {
    let mut r#type = None;
    let mut data = None;
    for (key, value) in event {
        match key.as_str() {
            "type" => r#type = Some(value),
            "data" => data = Some(value),
            _ => {}
        }
    }
    if let Some(r#type) = r#type {
        Some(Event::data(data.unwrap_or_default()).event(r#type).id(id))
    } else {
        None
    }
}
