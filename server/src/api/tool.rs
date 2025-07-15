use rocket::{
    delete,
    futures::future::{join_all, try_join},
    get, post,
    serde::json::Json,
    Route, State,
};
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
};
use schemars::JsonSchema;
use uuid::Uuid;

use crate::{
    auth::ChatRsUserId,
    db::{
        models::{
            ChatRsExecutedToolCall, ChatRsMessage, ChatRsMessageMeta, ChatRsMessageRole,
            ChatRsTool, ChatRsToolData, ChatRsToolJsonSchema, NewChatRsMessage, NewChatRsTool,
        },
        services::{ChatDbService, ToolDbService},
        DbConnection,
    },
    errors::ApiError,
    tools::{get_tool_response, validate_tool_schema, ChatRsToolError},
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
) -> Result<Json<Vec<ChatRsTool>>, ApiError> {
    let tools = ToolDbService::new(&mut db).find_by_user(&user_id).await?;

    Ok(Json(tools))
}

#[derive(JsonSchema, serde::Deserialize)]
struct ToolInput {
    /// Name of the tool
    name: String,
    /// Description of the tool
    description: String,
    /// JSON Schema of the tool's input parameters
    input_schema: ChatRsToolJsonSchema,
    /// Tool-specific data and configuration
    data: ChatRsToolData,
}

/// Create a new tool
#[openapi(tag = "Tools")]
#[post("/", data = "<input>")]
async fn create_tool(
    user_id: ChatRsUserId,
    mut db: DbConnection,
    mut input: Json<ToolInput>,
) -> Result<Json<ChatRsTool>, ApiError> {
    validate_tool_schema(&mut input.input_schema)?;

    let tool = ToolDbService::new(&mut db)
        .create(NewChatRsTool {
            user_id: &user_id,
            name: &input.name,
            description: &input.description,
            input_schema: &input.input_schema,
            data: &input.data,
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
) -> Result<Json<ChatRsMessage>, ApiError> {
    // Find message, tool call, and tool
    let message = ChatDbService::new(&mut db)
        .find_message(&user_id, &message_id)
        .await?;
    let tool_call = message
        .meta
        .tool_calls
        .and_then(|tool_calls| {
            tool_calls
                .into_iter()
                .find(|tool_call| tool_call.id == tool_call_id)
        })
        .ok_or(ChatRsToolError::ToolCallNotFound)?;
    let tool = ToolDbService::new(&mut db)
        .find_by_id(&user_id, &tool_call.tool_id)
        .await?
        .ok_or(ChatRsToolError::ToolNotFound)?;

    // Validate input parameters and get tool response
    let (content, is_error) = get_tool_response(&tool, &tool_call.parameters, http_client).await;

    let tool_message = ChatDbService::new(&mut db)
        .save_message(NewChatRsMessage {
            session_id: &message.session_id,
            role: ChatRsMessageRole::Tool,
            content: &content,
            meta: ChatRsMessageMeta {
                executed_tool_call: Some(ChatRsExecutedToolCall {
                    id: tool_call.id,
                    tool_id: tool_call.tool_id,
                    tool_name: tool_call.tool_name,
                    parameters: tool_call.parameters,
                    is_error,
                }),
                ..Default::default()
            },
        })
        .await?;

    Ok(Json(tool_message))
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
        .tool_calls
        .ok_or(ChatRsToolError::ToolCallNotFound)?;

    let tool_futures = tool_calls
        .iter()
        .filter_map(|tc| {
            let tool = user_tools.iter().find(|tool| tool.id == tc.tool_id)?;
            Some(get_tool_response(&tool, &tc.parameters, http_client))
        })
        .collect::<Vec<_>>();
    let tool_responses = join_all(tool_futures).await;

    let new_tool_messages = tool_responses
        .iter()
        .zip(tool_calls)
        .map(|(response, tool_call)| NewChatRsMessage {
            session_id: &message.session_id,
            role: ChatRsMessageRole::Tool,
            content: &response.0,
            meta: ChatRsMessageMeta {
                executed_tool_call: Some(ChatRsExecutedToolCall {
                    id: tool_call.id,
                    tool_id: tool_call.tool_id,
                    tool_name: tool_call.tool_name,
                    parameters: tool_call.parameters,
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
