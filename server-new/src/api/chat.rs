use aide::axum::ApiRouter;
use axum::{Json, extract::State};
use axum_typed_routing::{TypedApiRouter, api_route};
use derive_more::Into;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    api::ApiTag,
    error::{AppError, AppResult},
    extractors::{CurrentUser, Database},
    llm::types::{LlmChatOptions, LlmUserMessage},
    state::AppState,
};

pub fn routes() -> ApiRouter<AppState> {
    ApiRouter::new()
        .typed_api_route(prompt)
        .typed_api_route(chat_stream)
}

#[derive(Debug, Deserialize, JsonSchema)]
struct PromptInput {
    /// The prompt to send to the LLM provider
    message: String,
    /// The ID of the provider to chat with
    provider_id: i32,
    /// Configuration for the provider
    options: LlmChatOptions,
}

#[api_route(POST "/prompt" {
    summary: "Prompt",
    description: "Send a simple prompt to a provider and get the response",
    transform: |op| op.tag(ApiTag::Chat.into()),
})]
async fn prompt(
    CurrentUser { user_id }: CurrentUser,
    Database(mut db): Database,
    State(state): State<AppState>,
    Json(input): Json<PromptInput>,
) -> AppResult<Json<PromptResponse>> {
    let llm_provider = state
        .provider_service()
        .build_llm_provider(&mut db, &user_id, input.provider_id)
        .await?;
    let text = state
        .chat_service()
        .prompt(
            llm_provider,
            LlmUserMessage {
                text: input.message,
                ..Default::default()
            },
            input.options,
        )
        .await?;

    Ok(Json(PromptResponse { text }))
}

#[derive(Serialize, JsonSchema)]
struct PromptResponse {
    text: String,
}

#[derive(Into, Deserialize, JsonSchema)]
struct SessionIdPath(Uuid);

#[derive(Debug, Deserialize, JsonSchema)]
struct ChatInput {
    /// The new chat message from the user
    message: Option<String>,
    /// The ID of the provider to chat with
    provider_id: i32,
    /// Configuration for the provider
    options: LlmChatOptions,
}

#[api_route(POST "/{session_id}" {
    summary: "Streaming chat",
    description: "Send a message in a chat session and stream the response",
    transform: |op| op.tag(ApiTag::Chat.into()),
})]
async fn chat_stream(
    session_id: SessionIdPath,
    CurrentUser { user_id }: CurrentUser,
    Database(mut db): Database,
    State(state): State<AppState>,
    Json(input): Json<ChatInput>,
) -> Result<Json<StreamAccess>, AppError> {
    let llm_provider = state
        .provider_service()
        .build_llm_provider(&mut db, &user_id, input.provider_id)
        .await?;
    let stream_access = state
        .chat_service()
        .stream_user_chat(
            &mut db,
            user_id,
            session_id.into(),
            input.provider_id,
            llm_provider,
            input
                .message
                .map(|text| LlmUserMessage { text, files: None }),
            input.options,
        )
        .await?;

    Ok(Json(StreamAccess {
        url: stream_access.sse_url,
        token: stream_access.token,
    }))
}

/// The URL and Bearer token to access the SSE stream
#[derive(Serialize, JsonSchema)]
struct StreamAccess {
    url: String,
    token: String,
}
