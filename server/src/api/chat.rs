use llm::builder::LLMBackend;
use rocket::{
    futures::{Stream, StreamExt},
    post,
    response::stream::{Event, EventStream},
    routes, FromFormField, State,
};
use schemars::JsonSchema;
use uuid::Uuid;

use crate::{
    cached_stream::CachedStream,
    config::AppConfig,
    db::{
        models::{ChatRsMessageRole, NewChatRsMessage},
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
};

/// These are normal Rocket routes, not OpenAPI (`rocket_okapi` crate not working well with streams)
pub fn get_routes() -> impl Into<Vec<rocket::Route>> {
    routes![send_chat_stream]
}

#[derive(FromFormField, JsonSchema)]
pub enum ProviderInput {
    Lorem,
    Anthropic,
}

// #[openapi] // TODO this doesn't work with rocket_okapi macro - doesn't like EventStream wrapped in Result :(
#[post("/<session_id>?<message>&<provider>")]
pub async fn send_chat_stream(
    config: &State<AppConfig>,
    db_pool: &State<DbPool>,
    mut db: DbConnection,
    redis: RedisClient,
    session_id: Uuid,
    message: &str,
    provider: ProviderInput,
) -> Result<EventStream<impl Stream<Item = Event>>, ApiError> {
    let mut db_service = ChatDbService::new(&mut db);
    let (_current_session, current_messages) = db_service.get_session(&session_id).await?;
    let _ = db_service
        .save_message(NewChatRsMessage {
            content: message,
            session_id: &session_id,
            role: ChatRsMessageRole::User,
        })
        .await?;

    let provider: Box<dyn ChatRsProvider + Send> = match provider {
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

    let stream = provider.chat_stream(message, Some(current_messages)).await;
    let stream_with_caching = CachedStream::new(
        stream,
        db_pool.inner().clone(),
        redis.clone(),
        Some(session_id),
    );
    let event_stream = stream_with_caching.map(|result| match result {
        Ok(message) => Event::data(message).event("chat"),
        Err(err) => Event::data(err.to_string()).event("error"),
    });

    Ok(EventStream::from(event_stream))
}
