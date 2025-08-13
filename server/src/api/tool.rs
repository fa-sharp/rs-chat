use std::pin::Pin;

use rocket::{
    delete,
    futures::{
        future::{join_all, try_join},
        Stream,
    },
    get, post,
    response::stream::{Event, EventStream},
    serde::json::Json,
    Route, State,
};
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
};
use schemars::JsonSchema;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use uuid::Uuid;

use crate::{
    auth::ChatRsUserId,
    db::{
        models::{
            ChatRsExecutedToolCall, ChatRsMessage, ChatRsMessageMeta, ChatRsMessageRole,
            ChatRsToolPublic, NewChatRsMessage, NewChatRsTool,
        },
        services::{ChatDbService, ToolDbService},
        DbConnection,
    },
    errors::ApiError,
    tools::{ToolConfig, ToolError},
};

pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings:
        get_all_tools,
        execute_tool,
        execute_all_tools,
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

/// Execute a specific tool call in a message
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

    // Execute tool and save message
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    let http_client = http_client.inner().clone();
    tokio::spawn(async move {
        let (content, is_error) = tool.execute(&tool_call.parameters, &http_client, tx).await;
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
                    }),
                    ..Default::default()
                },
            })
            .await;
    });

    let stream: Pin<Box<dyn Stream<Item = Event> + Send>> =
        Box::pin(ReceiverStream::new(rx).map(|chunk| chunk.into()));
    Ok(EventStream::from(stream))
}

/// Execute all tool calls in a message
#[openapi(tag = "Tools")]
#[post("/execute/<message_id>")]
async fn execute_all_tools(
    user_id: ChatRsUserId,
    mut db_1: DbConnection,
    mut db_2: DbConnection,
    http_client: &State<reqwest::Client>,
    message_id: Uuid,
) -> Result<Json<Vec<ChatRsMessage>>, ApiError> {
    let (message, user_tools) = try_join(
        ChatDbService::new(&mut db_1).find_message(&user_id, &message_id),
        ToolDbService::new(&mut db_2).find_by_user(&user_id),
    )
    .await?;
    let tool_calls = message
        .meta
        .assistant
        .and_then(|meta| meta.tool_calls)
        .ok_or(ToolError::ToolCallNotFound)?;

    let (tx, rx) = tokio::sync::mpsc::channel(100);
    let tool_response_futures = tool_calls
        .iter()
        .map(
            async |tc| match user_tools.iter().find(|tool| tool.id == tc.tool_id) {
                Some(tool) => tool.execute(&tc.parameters, http_client, tx.clone()).await,
                None => (ToolError::ToolNotFound.to_string(), Some(true)),
            },
        )
        .collect::<Vec<_>>();
    let tool_responses = join_all(tool_response_futures).await;

    let new_tool_messages = tool_responses
        .iter()
        .zip(tool_calls)
        .map(|(response, tool_call)| NewChatRsMessage {
            session_id: &message.session_id,
            role: ChatRsMessageRole::Tool,
            content: &response.0,
            meta: ChatRsMessageMeta {
                tool_call: Some(ChatRsExecutedToolCall {
                    id: tool_call.id,
                    tool_id: tool_call.tool_id,
                    is_error: response.1.clone(),
                }),
                ..Default::default()
            },
        })
        .collect::<Vec<_>>();
    let saved_tool_messages = ChatDbService::new(&mut db_1)
        .save_messages(&new_tool_messages)
        .await?;

    Ok(Json(saved_tool_messages))
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
