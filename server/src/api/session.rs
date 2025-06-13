use rocket::{get, post, serde::json::Json, Route};
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
};
use schemars::JsonSchema;
use uuid::Uuid;

use crate::{
    db::{
        models::{ChatRsMessage, ChatRsSession, NewChatRsSession},
        services::chat::ChatDbService,
        DbConnection,
    },
    errors::ApiError,
};

pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: get_all_sessions, create_session, get_session]
}

#[derive(JsonSchema, serde::Serialize)]
struct CreateSessionResponse {
    session_id: String,
}

#[openapi]
#[get("/")]
async fn get_all_sessions(mut db: DbConnection) -> Result<Json<Vec<ChatRsSession>>, ApiError> {
    let sessions = ChatDbService::new(&mut db).get_all_sessions().await?;

    Ok(Json(sessions))
}

#[openapi]
#[post("/")]
async fn create_session(mut db: DbConnection) -> Result<Json<CreateSessionResponse>, ApiError> {
    let id = ChatDbService::new(&mut db)
        .create_session(NewChatRsSession { title: "New Chat" })
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
    mut db: DbConnection,
    session_id: Uuid,
) -> Result<Json<GetSessionResponse>, ApiError> {
    let (session, messages) = ChatDbService::new(&mut db).get_session(&session_id).await?;

    Ok(Json(GetSessionResponse { session, messages }))
}
