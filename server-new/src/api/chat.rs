use aide_docs_macro::api_routes;
use axum::{
    Json,
    extract::{Path, State},
};
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

api_routes! {
    state: AppState,
    tag: ApiTag::Chat,
    POST "/prompt" => prompt, "Prompt", {
        description: "Send a single prompt to a provider and get the response"
    };
    POST "/session/{session_id}" => chat_stream, "Chat", {
        description: "Send a message in a chat session and stream the response"
    };
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

#[derive(Debug, Deserialize, JsonSchema)]
struct ChatInput {
    /// The new chat message from the user
    message: Option<String>,
    /// The ID of the provider to chat with
    provider_id: i32,
    /// Configuration for the provider
    options: LlmChatOptions,
}

async fn chat_stream(
    Path(session_id): Path<Uuid>,
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
            session_id,
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
