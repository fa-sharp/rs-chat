use axum::{
    Json,
    extract::{Path, State},
};
use axum_aide_macros::api_routes;
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
    tag: ApiTag::Chat.into(),
    POST "/prompt" => prompt, "Prompt";
    GET "/session" => get_active_streams, "Get sessions with active streams";
    GET "/session/{session_id}" => connect_chat_stream, "Access active chat stream";
    POST "/session/{session_id}" => chat_stream, "Stream chat session response";
    POST "/session/{session_id}/cancel" => cancel_chat_stream, "Cancel active chat stream";
    POST "/session/{session_id}/regenerate" => regenerate_response, "Regenerate chat response";
}

async fn get_active_streams(
    CurrentUser { user_id }: CurrentUser,
    State(state): State<AppState>,
) -> AppResult<Json<ActiveStreamsResponse>> {
    let sessions = state
        .chat_service()
        .active_stream_sessions(&user_id)
        .await?;

    Ok(Json(ActiveStreamsResponse { sessions }))
}

async fn prompt(
    CurrentUser { user_id }: CurrentUser,
    Database(mut db): Database,
    State(state): State<AppState>,
    Json(PromptInput {
        message,
        provider_id,
        options,
    }): Json<PromptInput>,
) -> AppResult<Json<StreamAccess>> {
    let llm_provider = state
        .provider_service()
        .build_llm_provider(&mut db, &user_id, provider_id)
        .await?;
    let prompt = LlmUserMessage {
        text: message,
        ..Default::default()
    };
    let stream_access = state
        .chat_service()
        .prompt(&mut db, user_id, provider_id, llm_provider, prompt, options)
        .await?;

    Ok(Json(StreamAccess {
        url: stream_access.sse_url,
        token: stream_access.token,
    }))
}

async fn chat_stream(
    CurrentUser { user_id }: CurrentUser,
    Path(session_id): Path<Uuid>,
    Database(mut db): Database,
    State(state): State<AppState>,
    Json(input): Json<ChatInput>,
) -> Result<Json<StreamAccess>, AppError> {
    let llm_provider = state
        .provider_service()
        .build_llm_provider(&mut db, &user_id, input.provider_id)
        .await?;
    let user_message = input
        .message
        .map(|text| LlmUserMessage { text, files: None });
    let stream_access = state
        .chat_service()
        .stream_user_chat(
            &mut db,
            user_id,
            session_id,
            input.provider_id,
            llm_provider,
            user_message,
            input.options,
        )
        .await?;

    Ok(Json(StreamAccess {
        url: stream_access.sse_url,
        token: stream_access.token,
    }))
}

async fn regenerate_response(
    CurrentUser { user_id }: CurrentUser,
    Path(session_id): Path<Uuid>,
    Database(mut db): Database,
    State(state): State<AppState>,
    Json(input): Json<RegenerateInput>,
) -> Result<Json<StreamAccess>, AppError> {
    let llm_provider = state
        .provider_service()
        .build_llm_provider(&mut db, &user_id, input.provider_id)
        .await?;
    let stream_access = state
        .chat_service()
        .regenerate_response(
            &mut db,
            user_id,
            session_id,
            input.provider_id,
            llm_provider,
            input.options,
        )
        .await?;

    Ok(Json(StreamAccess {
        url: stream_access.sse_url,
        token: stream_access.token,
    }))
}

async fn connect_chat_stream(
    CurrentUser { user_id }: CurrentUser,
    Path(session_id): Path<Uuid>,
    State(state): State<AppState>,
) -> AppResult<Json<StreamAccess>> {
    let stream_access = state
        .chat_service()
        .connect_stream(&user_id, &session_id)
        .await?;

    Ok(Json(StreamAccess {
        url: stream_access.sse_url,
        token: stream_access.token,
    }))
}

pub async fn cancel_chat_stream(
    CurrentUser { user_id }: CurrentUser,
    Path(session_id): Path<Uuid>,
    State(state): State<AppState>,
) -> AppResult<()> {
    state
        .chat_service()
        .cancel_stream(&user_id, &session_id)
        .await?;

    Ok(())
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

#[derive(Debug, Deserialize, JsonSchema)]
struct ChatInput {
    /// The new chat message from the user
    message: Option<String>,
    /// The ID of the provider to chat with
    provider_id: i32,
    /// Configuration for the provider
    options: LlmChatOptions,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct RegenerateInput {
    /// The ID of the provider to chat with
    provider_id: i32,
    /// Configuration for the provider
    options: LlmChatOptions,
}

#[derive(Debug, JsonSchema, serde::Serialize)]
struct ActiveStreamsResponse {
    /// The chat session IDs that have ongoing response streams
    sessions: Vec<Uuid>,
}

/// Access to an active streaming response
#[derive(Serialize, JsonSchema)]
struct StreamAccess {
    /// URL to access the SSE stream
    url: String,
    /// Bearer token to access the SSE stream
    token: String,
}
