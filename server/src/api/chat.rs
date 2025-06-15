use std::{borrow::Cow, pin::Pin};

use llm::builder::LLMBackend;
use rocket::{
    futures::{stream::once, Stream, StreamExt},
    post,
    response::stream::{Event, EventStream},
    serde::json::Json,
    FromFormField, Route, State,
};
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
};
use schemars::JsonSchema;
use uuid::Uuid;

use crate::{
    config::AppConfig,
    db::{
        models::{ChatRsMessageRole, ChatRsUser, NewChatRsMessage},
        services::chat::ChatDbService,
        DbConnection, DbPool,
    },
    errors::ApiError,
    provider::{
        llm::LlmApiProvider,
        lorem::{LoremConfig, LoremProvider},
        ChatRsProvider,
    },
    redis::RedisClient,
    utils::stored_stream::StoredChatRsStream,
};

pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: send_chat_stream]
}

#[derive(FromFormField, JsonSchema, serde::Deserialize)]
pub enum ProviderInput {
    Lorem,
    Anthropic,
}

#[derive(JsonSchema, serde::Deserialize)]
pub struct SendChatInput<'a> {
    message: Option<Cow<'a, str>>,
    provider: ProviderInput,
}

#[openapi]
#[post("/<session_id>", data = "<input>")]
pub async fn send_chat_stream(
    user: ChatRsUser,
    config: &State<AppConfig>,
    db_pool: &State<DbPool>,
    mut db: DbConnection,
    redis: RedisClient,
    session_id: Uuid,
    input: Json<SendChatInput<'_>>,
) -> Result<EventStream<Pin<Box<dyn Stream<Item = Event> + Send>>>, ApiError> {
    let mut db_service = ChatDbService::new(&mut db);

    // Check session exists and user is owner, get message history
    let (_, current_messages) = db_service.get_session(&user.id, &session_id).await?;

    // Save user message to session
    if let Some(user_message) = &input.message {
        let _ = db_service
            .save_message(NewChatRsMessage {
                content: user_message,
                session_id: &session_id,
                role: ChatRsMessageRole::User,
            })
            .await?;
    }

    // Get the chat provider
    let provider: Box<dyn ChatRsProvider + Send> = match input.provider {
        ProviderInput::Lorem => Box::new(LoremProvider {
            config: LoremConfig { interval: 400 },
        }),
        ProviderInput::Anthropic => Box::new(LlmApiProvider::new(
            LLMBackend::Anthropic,
            &config.anthropic_api_key,
            "claude-3-7-sonnet-20250219",
            None,
            None,
        )),
    };

    // Stream the provider's response
    let stream = StoredChatRsStream::new(
        provider
            .chat_stream(input.message.as_deref(), Some(current_messages))
            .await?,
        db_pool.inner().clone(),
        redis.clone(),
        Some(session_id),
    );
    let event_stream: Pin<Box<dyn Stream<Item = Event> + Send>> = Box::pin(
        stream
            .map(|result| match result {
                Ok(message) => Event::data(format!(" {message}")).event("chat"),
                Err(err) => Event::data(err.to_string()).event("error"),
            })
            .chain(once(async { Event::empty().event("end") })),
    );

    Ok(EventStream::from(event_stream))
}
