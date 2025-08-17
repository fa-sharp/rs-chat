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
    api::session::DEFAULT_SESSION_TITLE,
    auth::ChatRsUserId,
    db::{
        models::{
            ChatRsMessageMeta, ChatRsMessageRole, ChatRsProviderType, ChatRsSessionMeta,
            NewChatRsMessage, UpdateChatRsSession,
        },
        services::{ChatDbService, ProviderDbService, ToolDbService},
        DbConnection, DbPool,
    },
    errors::ApiError,
    provider::{build_llm_provider_api, LlmApiProviderSharedOptions, LlmTool},
    redis::RedisClient,
    tools::SendChatToolInput,
    utils::{
        encryption::Encryptor, generate_title::generate_title, stored_stream::StoredChatRsStream,
    },
};

pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: send_chat_stream]
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
