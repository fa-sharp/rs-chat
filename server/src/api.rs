use llm::builder::LLMBackend;
use rocket::{
    futures::{Stream, StreamExt},
    post,
    response::stream::{Event, EventStream},
    FromFormField, Route, State,
};
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
};
use schemars::JsonSchema;

use crate::{
    config::AppConfig,
    provider::{
        llm::LlmApiProvider,
        lorem::{LoremChatProvider, LoremConfig},
        ChatProvider,
    },
};

pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: chat]
}

#[derive(FromFormField, JsonSchema)]
enum ProviderInput {
    Lorem,
    Anthropic,
}

#[openapi]
#[post("/chat?<message>&<provider>")]
async fn chat(
    config: &State<AppConfig>,
    message: &str,
    provider: ProviderInput,
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

    let event_stream = provider
        .chat_stream(message, None)
        .await
        .map(|result| match result {
            Ok(message) => Event::data(message).event("chat"),
            Err(err) => Event::data(err.to_string()).event("error"),
        });

    EventStream::from(event_stream)
}
