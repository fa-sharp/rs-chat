use std::collections::HashMap;

use rocket::{delete, get, post, serde::json::Json, Route, State};
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
    tools::{ChatRsToolError, HttpRequestTool},
};

pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: get_all_tools, execute_tool, create_tool, delete_tool]
}

/// List all tools
#[openapi(tag = "Tools")]
#[get("/")]
async fn get_all_tools(
    user_id: ChatRsUserId,
    mut db: DbConnection,
) -> Result<Json<Vec<ChatRsTool>>, ApiError> {
    let tools = ToolDbService::new(&mut db)
        .find_by_user_id(&user_id)
        .await?;

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

/// Ensure JSON schema is valid (using Draft 2020-12).
/// Also sets `additionalProperties` to false as required by OpenAI.
fn validate_tool_schema(input_schema: &mut ChatRsToolJsonSchema) -> Result<(), ChatRsToolError> {
    input_schema.additional_properties = Some(false);
    jsonschema::draft202012::meta::validate(&serde_json::to_value(input_schema)?)
        .map_err(|e| ChatRsToolError::InvalidJsonSchema(e.to_string()))?;
    Ok(())
}

/// Execute a tool call
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
    let (content, is_error) = {
        if let Err(e) = validate_tool_input(&tool.input_schema, &tool_call.parameters) {
            (e.to_string(), Some(true))
        } else {
            match tool.data {
                ChatRsToolData::Http(http_request_config) => {
                    let http_request_tool = HttpRequestTool::new(http_client, http_request_config);
                    http_request_tool
                        .execute_tool(&tool_call.parameters)
                        .await
                        .map_or_else(|e| (e.to_string(), Some(true)), |response| (response, None))
                }
            }
        }
    };

    let tool_message = ChatDbService::new(&mut db)
        .save_message(NewChatRsMessage {
            session_id: &message.session_id,
            role: ChatRsMessageRole::Tool,
            content: &content,
            meta: &ChatRsMessageMeta {
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

fn validate_tool_input(
    input_schema: &ChatRsToolJsonSchema,
    parameters: &HashMap<String, serde_json::Value>,
) -> Result<(), ChatRsToolError> {
    jsonschema::validate(
        &serde_json::to_value(input_schema)?,
        &serde_json::to_value(parameters)?,
    )
    .map_err(|err| ChatRsToolError::InvalidParameters(err.to_string()))
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
