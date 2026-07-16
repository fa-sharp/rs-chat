use std::borrow::Cow;

use axum::{
    Json,
    extract::{Path, Query},
};
use axum_aide_macros::api_routes;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    api::ApiTag,
    db::{
        models::{
            ChatRsLogLlmRequest, ChatRsMessage, ChatRsSession, NewChatRsSession,
            UpdateChatRsSession,
        },
        queries::FullTextSearchResult,
    },
    error::{AppError, AppResult},
    extractors::{CurrentUser, Database},
    services::chat::DEFAULT_SESSION_TITLE,
    state::AppState,
};

api_routes! {
    state: AppState,
    tag: ApiTag::Chat.into(),
    GET "/" => get_recent_sessions, "List recent chat sessions";
    POST "/" => create_session, "Create chat session";
    GET "/{session_id}" => get_session, "Get chat session";
    GET "/search" => search_sessions, "Search chat sessions";
    PATCH "/{session_id}" => update_session, "Update chat session";
    DELETE "/{session_id}" => delete_session, "Delete chat session";
    DELETE "/{session_id}/{message_id}" => delete_message, "Delete chat message";
}

async fn get_recent_sessions(
    CurrentUser { user_id }: CurrentUser,
    Database(mut db): Database,
) -> AppResult<Json<Vec<ChatRsSession>>> {
    let sessions = db.chats().get_recent_sessions(&user_id).await?;

    Ok(Json(sessions))
}

async fn create_session(
    CurrentUser { user_id }: CurrentUser,
    Database(mut db): Database,
) -> AppResult<Json<SessionIdResponse>> {
    let session_id = db
        .chats()
        .create_session(NewChatRsSession {
            user_id: &user_id,
            title: DEFAULT_SESSION_TITLE,
        })
        .await?;

    Ok(Json(SessionIdResponse { session_id }))
}

async fn get_session(
    CurrentUser { user_id }: CurrentUser,
    Database(mut db): Database,
    Path(session_id): Path<Uuid>,
) -> AppResult<Json<GetSessionResponse>> {
    let session = db
        .chats()
        .find_session(&user_id, &session_id)
        .await?
        .ok_or_else(|| AppError::not_found("chat session not found"))?;
    let messages = db
        .chats()
        .list_messages_with_logs(&session_id)
        .await?
        .into_iter()
        .map(|(message, llm_request)| SessionMessage {
            message,
            llm_request,
        })
        .collect();

    Ok(Json(GetSessionResponse { session, messages }))
}

async fn search_sessions(
    CurrentUser { user_id }: CurrentUser,
    Database(mut db): Database,
    Query(SessionSearchQuery { query }): Query<SessionSearchQuery<'_>>,
) -> AppResult<Json<Vec<FullTextSearchResult>>> {
    let sessions = db.chats().search_sessions(&user_id, &query).await?;

    Ok(Json(sessions))
}

async fn update_session(
    CurrentUser { user_id }: CurrentUser,
    Database(mut db): Database,
    Path(session_id): Path<Uuid>,
    Json(input): Json<UpdateSessionInput>,
) -> AppResult<Json<SessionIdResponse>> {
    let updated_id = db
        .chats()
        .update_session(
            &user_id,
            &session_id,
            UpdateChatRsSession {
                title: Some(&input.title),
                ..Default::default()
            },
        )
        .await?;

    match updated_id {
        Some(session_id) => Ok(Json(SessionIdResponse { session_id })),
        None => Err(AppError::not_found("chat session not found")),
    }
}

async fn delete_session(
    CurrentUser { user_id }: CurrentUser,
    Database(mut db): Database,
    Path(session_id): Path<Uuid>,
) -> AppResult<Json<SessionIdResponse>> {
    match db.chats().delete_session(&user_id, &session_id).await? {
        Some(session_id) => Ok(Json(SessionIdResponse { session_id })),
        None => Err(AppError::not_found("chat session not found")),
    }
}

async fn delete_message(
    CurrentUser { user_id }: CurrentUser,
    Database(mut db): Database,
    Path((session_id, message_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<MessageIdResponse>> {
    match db.chats().find_session(&user_id, &session_id).await? {
        Some(session) => match db.chats().delete_message(&session.id, &message_id).await? {
            Some(message_id) => Ok(Json(MessageIdResponse { message_id })),
            None => Err(AppError::not_found("chat message not found")),
        },
        None => Err(AppError::not_found("chat session not found")),
    }
}

#[derive(Serialize, JsonSchema)]
struct SessionIdResponse {
    session_id: Uuid,
}

#[derive(Serialize, JsonSchema)]
struct MessageIdResponse {
    message_id: Uuid,
}

#[derive(Serialize, JsonSchema)]
struct GetSessionResponse {
    session: ChatRsSession,
    messages: Vec<SessionMessage>,
}

#[derive(Serialize, JsonSchema)]
struct SessionMessage {
    /// The message
    message: ChatRsMessage,
    /// Request metadata for assistant responses
    llm_request: Option<ChatRsLogLlmRequest>,
}

#[derive(Deserialize, JsonSchema)]
struct SessionSearchQuery<'q> {
    query: Cow<'q, str>,
}

#[derive(Deserialize, JsonSchema)]
struct UpdateSessionInput {
    title: String,
}
