use fred::prelude::KeysInterface;
use llm::builder::LLMBackend;
use rocket::{
    futures::{Stream, StreamExt},
    get, post,
    response::stream::{Event, EventStream},
    FromFormField, Route, State,
};
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
};
use schemars::JsonSchema;

use crate::{
    cached_stream::CachedStream,
    config::AppConfig,
    provider::{
        llm::LlmApiProvider,
        lorem::{LoremChatProvider, LoremConfig},
        ChatProvider,
    },
    redis::RedisClient,
};

pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: chat, resume_chat]
}

#[derive(FromFormField, JsonSchema)]
enum ProviderInput {
    Lorem,
    Anthropic,
}

#[openapi]
#[post("/chat?<message>&<provider>&<session_id>")]
async fn chat(
    config: &State<AppConfig>,
    redis: RedisClient,
    message: &str,
    provider: ProviderInput,
    session_id: Option<String>,
) -> EventStream<impl Stream<Item = Event>> {
    let provider: Box<dyn ChatProvider + Send> = match provider {
        ProviderInput::Lorem => Box::new(LoremChatProvider {
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

    let stream = provider.chat_stream(message, None).await;
    let stream_with_caching = CachedStream::new(stream, redis.clone(), session_id);
    let event_stream = stream_with_caching.map(|result| match result {
        Ok(message) => Event::data(message).event("chat"),
        Err(err) => Event::data(err.to_string()).event("error"),
    });

    EventStream::from(event_stream)
}

#[openapi]
#[get("/chat/resume/<session_id>")]
async fn resume_chat(redis: RedisClient, session_id: &str) -> Result<String, &'static str> {
    let cached_content: Option<String> = redis
        .get(format!("chat_session:{}", session_id))
        .await
        .map_err(|_| "Failed to retrieve cached content")?;

    cached_content.ok_or("Session not found or expired")
}
