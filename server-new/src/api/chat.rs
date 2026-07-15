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
    POST "/prompt" => prompt, "Prompt", {
        description: "Send a single prompt to a provider and get the response"
    };
    GET "/sessions" => get_active_streams, "Get active chat streams", {
        description: "Get the session IDs that have ongoing response streams"
    };
    GET "/sessions/{session_id}" => connect_chat_stream, "Access active chat stream", {
        description: "Get a URL and token to access the response stream for this session"
    };
    POST "/sessions/{session_id}" => chat_stream, "Stream chat", {
        description: "Send a message in a chat session and stream the response"
    };
    POST "/sessions/{session_id}/cancel" => cancel_chat_stream, "Cancel active stream", {
        description: "Cancel an ongoing chat stream"
    };
    POST "/session/{session_id}/regenerate" => regenerate_response, "Regenerate chat response", {
        description: "Regenerate the latest assistant response in a chat session"
    };
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

#[derive(Debug, JsonSchema, serde::Serialize)]
struct ActiveStreamsResponse {
    /// The chat session IDs that have ongoing response streams
    sessions: Vec<Uuid>,
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
) -> AppResult<Json<StreamAccess>> {
    let llm_provider = state
        .provider_service()
        .build_llm_provider(&mut db, &user_id, input.provider_id)
        .await?;
    let stream_access = state
        .chat_service()
        .prompt(
            &mut db,
            user_id,
            input.provider_id,
            llm_provider,
            LlmUserMessage {
                text: input.message,
                ..Default::default()
            },
            input.options,
        )
        .await?;

    Ok(Json(StreamAccess {
        url: stream_access.sse_url,
        token: stream_access.token,
    }))
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

#[derive(Debug, Deserialize, JsonSchema)]
struct RegenerateInput {
    /// The ID of the provider to chat with
    provider_id: i32,
    /// Configuration for the provider
    options: LlmChatOptions,
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

/// Access to an active streaming response
#[derive(Serialize, JsonSchema)]
struct StreamAccess {
    /// URL to access the SSE stream
    url: String,
    /// Bearer token to access the SSE stream
    token: String,
}
