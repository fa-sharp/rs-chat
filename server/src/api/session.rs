use rocket::{delete, get, post, serde::json::Json, Route};
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
};
use schemars::JsonSchema;
use uuid::Uuid;

use crate::{
    db::{
        models::{ChatRsMessage, ChatRsSession, ChatRsUser, NewChatRsSession},
        services::chat::ChatDbService,
        DbConnection,
    },
    errors::ApiError,
};

pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: get_all_sessions, create_session, get_session, delete_message]
}

#[derive(JsonSchema, serde::Serialize)]
struct CreateSessionResponse {
    session_id: String,
}

#[openapi]
#[get("/")]
async fn get_all_sessions(
    user: ChatRsUser,
    mut db: DbConnection,
) -> Result<Json<Vec<ChatRsSession>>, ApiError> {
    let sessions = ChatDbService::new(&mut db)
        .get_all_sessions(&user.id)
        .await?;

    Ok(Json(sessions))
}

#[openapi]
#[post("/")]
async fn create_session(
    user: ChatRsUser,
    mut db: DbConnection,
) -> Result<Json<CreateSessionResponse>, ApiError> {
    let id = ChatDbService::new(&mut db)
        .create_session(NewChatRsSession {
            user_id: &user.id,
            title: "New Chat",
        })
        .await?;

    Ok(Json(CreateSessionResponse { session_id: id }))
}

#[derive(JsonSchema, serde::Serialize)]
struct GetSessionResponse {
    session: ChatRsSession,
    messages: Vec<ChatRsMessage>,
}

#[openapi]
#[get("/<session_id>")]
async fn get_session(
    user: ChatRsUser,
    mut db: DbConnection,
    session_id: Uuid,
) -> Result<Json<GetSessionResponse>, ApiError> {
    let (session, messages) = ChatDbService::new(&mut db)
        .get_session_with_messages(&user.id, &session_id)
        .await?;

    Ok(Json(GetSessionResponse { session, messages }))
}

#[openapi]
#[delete("/<session_id>/<message_id>")]
async fn delete_message(
    user: ChatRsUser,
    mut db: DbConnection,
    session_id: Uuid,
    message_id: Uuid,
) -> Result<(), ApiError> {
    let mut db_service = ChatDbService::new(&mut db);
    let _ = db_service.get_session(&user.id, &session_id).await?;
    let _ = db_service.delete_message(&session_id, &message_id).await?;

    Ok(())
}
