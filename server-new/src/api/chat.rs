use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    error::AppError,
    extractors::{database::Database, session::UserSession},
    llm::types::{LlmChatOptions, LlmUserMessage},
    state::AppState,
};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(chat_stream))
}

#[derive(Debug, Deserialize)]
struct ChatInput {
    /// The new chat message from the user
    message: Option<String>,
    /// The ID of the provider to chat with
    provider_id: i32,
    /// Configuration for the provider
    options: LlmChatOptions,
}

#[utoipa::path(get, path = "/{session_id}", params(("session_id" = Uuid, Path)), responses((status = OK, body = StreamAccess)))]
async fn chat_stream(
    UserSession { user_id }: UserSession,
    Path(session_id): Path<Uuid>,
    Database(mut db): Database,
    State(state): State<AppState>,
    Json(input): Json<ChatInput>,
) -> Result<impl IntoResponse, AppError> {
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

#[derive(Serialize, ToSchema)]
struct StreamAccess {
    url: String,
    token: String,
}
