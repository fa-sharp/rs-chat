use fred::prelude::KeysInterface;
use rocket::{delete, get, patch, post, serde::json::Json, Route};
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
};
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    auth::ChatRsUserId,
    db::{
        models::{
            ChatRsMessage, ChatRsMessageMeta, ChatRsMessageRole, ChatRsSession, NewChatRsSession,
            UpdateChatRsSession,
        },
        services::chat::ChatDbService,
        DbConnection,
    },
    errors::ApiError,
    redis::RedisClient,
    utils::stored_stream::CACHE_KEY_PREFIX,
};

pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        settings: get_all_sessions,
        create_session,
        get_session,
        update_session,
        delete_session,
        delete_message
    ]
}

#[derive(JsonSchema, serde::Serialize)]
struct SessionIdResponse {
    session_id: String,
}

/// List chat sessions
#[openapi(tag = "Chat Session")]
#[get("/")]
async fn get_all_sessions(
    user_id: ChatRsUserId,
    mut db: DbConnection,
) -> Result<Json<Vec<ChatRsSession>>, ApiError> {
    let sessions = ChatDbService::new(&mut db)
        .get_all_sessions(&user_id)
        .await?;

    Ok(Json(sessions))
}

/// Create a new chat session
#[openapi(tag = "Chat Session")]
#[post("/")]
async fn create_session(
    user_id: ChatRsUserId,
    mut db: DbConnection,
) -> Result<Json<SessionIdResponse>, ApiError> {
    let id = ChatDbService::new(&mut db)
        .create_session(NewChatRsSession {
            user_id: &user_id,
            title: "New Chat",
        })
        .await?;

    Ok(Json(SessionIdResponse { session_id: id }))
}

#[derive(JsonSchema, serde::Serialize)]
struct GetSessionResponse {
    session: ChatRsSession,
    messages: Vec<ChatRsMessage>,
}

/// Get a chat session and its messages
#[openapi(tag = "Chat Session")]
#[get("/<session_id>")]
async fn get_session(
    user_id: ChatRsUserId,
    mut db: DbConnection,
    redis: RedisClient,
    session_id: Uuid,
) -> Result<Json<GetSessionResponse>, ApiError> {
    let (session, mut messages) = ChatDbService::new(&mut db)
        .get_session_with_messages(&user_id, &session_id)
        .await?;

    // Check for a cached response if the session is interrupted
    let cached_response: Option<String> = redis
        .get(format!("{}{}", CACHE_KEY_PREFIX, &session_id))
        .await?;
    if let Some(interrupted_response) = cached_response {
        messages.push(ChatRsMessage {
            id: Uuid::new_v4(),
            session_id,
            role: ChatRsMessageRole::Assistant,
            content: interrupted_response,
            created_at: chrono::Utc::now(),
            meta: ChatRsMessageMeta {
                interrupted: Some(true),
                ..Default::default()
            },
        });
    }

    Ok(Json(GetSessionResponse { session, messages }))
}

#[derive(Deserialize, JsonSchema)]
struct UpdateSessionInput {
    title: String,
}

/// Update chat session
#[openapi(tag = "Chat Session")]
#[patch("/<session_id>", data = "<body>")]
async fn update_session(
    user_id: ChatRsUserId,
    mut db: DbConnection,
    session_id: Uuid,
    body: Json<UpdateSessionInput>,
) -> Result<Json<SessionIdResponse>, ApiError> {
    let updated_id = ChatDbService::new(&mut db)
        .update_session(
            &user_id,
            &session_id,
            UpdateChatRsSession { title: &body.title },
        )
        .await?;

    Ok(Json(SessionIdResponse {
        session_id: updated_id.to_string(),
    }))
}

/// Delete a chat message
#[openapi(tag = "Chat Session")]
#[delete("/<session_id>/<message_id>")]
async fn delete_message(
    user_id: ChatRsUserId,
    mut db: DbConnection,
    session_id: Uuid,
    message_id: Uuid,
) -> Result<(), ApiError> {
    let mut db_service = ChatDbService::new(&mut db);
    let _ = db_service.get_session(&user_id, &session_id).await?;
    let _ = db_service.delete_message(&session_id, &message_id).await?;

    Ok(())
}

/// Delete chat session
#[openapi(tag = "Chat Session")]
#[delete("/<session_id>")]
async fn delete_session(
    user_id: ChatRsUserId,
    mut db: DbConnection,
    session_id: Uuid,
) -> Result<Json<SessionIdResponse>, ApiError> {
    let deleted_id = ChatDbService::new(&mut db)
        .delete_session(&user_id, &session_id)
        .await?;

    Ok(Json(SessionIdResponse {
        session_id: deleted_id.to_string(),
    }))
}
