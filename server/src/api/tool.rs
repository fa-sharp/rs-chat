use std::pin::Pin;

use rocket::{
    delete,
    futures::{Stream, StreamExt},
    get, post,
    response::stream::{Event, EventStream},
    serde::json::Json,
    Route, State,
};
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
};
use schemars::JsonSchema;
use tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;

use crate::{
    auth::ChatRsUserId,
    db::{
        models::{
            ChatRsExecutedToolCall, ChatRsMessageMeta, ChatRsMessageRole, ChatRsToolPublic,
            NewChatRsMessage, NewChatRsTool,
        },
        services::{ChatDbService, ToolDbService},
        DbConnection,
    },
    errors::ApiError,
    tools::{ToolConfig, ToolError, ToolLog},
    utils::sender_with_logging::SenderWithLogging,
};

pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings:
        get_all_tools,
        execute_tool,
        create_tool,
        delete_tool,
    ]
}

/// List all tools
#[openapi(tag = "Tools")]
#[get("/")]
async fn get_all_tools(
    user_id: ChatRsUserId,
    mut db: DbConnection,
) -> Result<Json<Vec<ChatRsToolPublic>>, ApiError> {
    let tools = ToolDbService::new(&mut db)
        .find_by_user_public(&user_id)
        .await?;

    Ok(Json(tools))
}

#[derive(JsonSchema, serde::Deserialize)]
struct ToolInput {
    /// Name of the tool
    name: String,
    /// Description of the tool
    description: String,
    /// Tool-specific configuration
    config: ToolConfig,
}

/// Create a new tool
#[openapi(tag = "Tools")]
#[post("/", data = "<input>")]
async fn create_tool(
    user_id: ChatRsUserId,
    mut db: DbConnection,
    mut input: Json<ToolInput>,
) -> Result<Json<ChatRsToolPublic>, ApiError> {
    input.config.validate()?;

    let tool = ToolDbService::new(&mut db)
        .create(NewChatRsTool {
            user_id: &user_id,
            name: &input.name,
            description: &input.description,
            config: &input.config,
        })
        .await?;

    Ok(Json(tool))
}

/// Execute a tool call and stream its output
#[openapi(tag = "Tools")]
#[post("/execute/<message_id>/<tool_call_id>")]
async fn execute_tool(
    user_id: ChatRsUserId,
    mut db: DbConnection,
    http_client: &State<reqwest::Client>,
    message_id: Uuid,
    tool_call_id: &str,
) -> Result<EventStream<Pin<Box<dyn Stream<Item = Event> + Send>>>, ApiError> {
    // Find message, tool call, and tool
    let message = ChatDbService::new(&mut db)
        .find_message(&user_id, &message_id)
        .await?;
    let tool_call = message
        .meta
        .assistant
        .and_then(|meta| meta.tool_calls)
        .and_then(|tool_calls| {
            tool_calls
                .into_iter()
                .find(|tool_call| tool_call.id == tool_call_id)
        })
        .ok_or(ToolError::ToolCallNotFound)?;
    let tool = ToolDbService::new(&mut db)
        .find_by_id(&user_id, &tool_call.tool_id)
        .await?
        .ok_or(ToolError::ToolNotFound)?;

    let (streaming_tx, streaming_rx) = tokio::sync::mpsc::channel(50);
    let http_client = http_client.inner().clone();

    // Spawn async tasks to collect logs, execute tool, and save final result to database
    tokio::spawn(async move {
        let (log_tx, mut log_rx) = tokio::sync::mpsc::channel(50);
        let sender_with_logging = SenderWithLogging::new(streaming_tx, log_tx);

        let log_collector_task = tokio::spawn(async move {
            let mut logs = None;
            let mut errors = None;
            while let Some(chunk) = log_rx.recv().await {
                match chunk {
                    ToolLog::Log(data) => logs
                        .get_or_insert_with(|| Vec::with_capacity(20))
                        .push(data),
                    ToolLog::Error(data) => errors
                        .get_or_insert_with(|| Vec::with_capacity(5))
                        .push(data),
                    _ => {}
                }
            }
            (logs, errors)
        });

        // Execute tool and collect logs
        let (content, is_error) = tool
            .execute(&tool_call.parameters, &http_client, &sender_with_logging)
            .await;
        drop(sender_with_logging); // Drop sender to close logging channel
        let (logs, errors) = log_collector_task.await.unwrap_or_default();

        // Save final result to database
        let _ = ChatDbService::new(&mut db)
            .save_message(NewChatRsMessage {
                session_id: &message.session_id,
                role: ChatRsMessageRole::Tool,
                content: &content,
                meta: ChatRsMessageMeta {
                    tool_call: Some(ChatRsExecutedToolCall {
                        id: tool_call.id,
                        tool_id: tool_call.tool_id,
                        is_error,
                        logs,
                        errors,
                    }),
                    ..Default::default()
                },
            })
            .await;
    });

    // Stream output
    let stream = ReceiverStream::new(streaming_rx)
        .map(|chunk| chunk.into())
        .boxed();
    Ok(EventStream::from(stream))
}

/// Delete a tool
#[openapi(tag = "Tools")]
#[delete("/<tool_id>")]
async fn delete_tool(
    user_id: ChatRsUserId,
    mut db: DbConnection,
    tool_id: Uuid,
) -> Result<String, ApiError> {
    let id = ToolDbService::new(&mut db)
        .delete(&user_id, &tool_id)
        .await?;

    Ok(id.to_string())
}
