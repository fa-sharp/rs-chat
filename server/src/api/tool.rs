use rocket::{delete, get, post, serde::json::Json, Route};
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
};
use schemars::JsonSchema;
use uuid::Uuid;

use crate::{
    auth::ChatRsUserId,
    db::{
        models::{ChatRsTool, NewChatRsTool},
        services::ToolDbService,
        DbConnection,
    },
    errors::ApiError,
};

pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: get_all_tools, create_tool, delete_tool]
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
    name: String,
    description: String,
    url: String,
    method: String,
    query: Option<serde_json::Value>,
    body: Option<serde_json::Value>,
}

/// Create a new tool
#[openapi(tag = "Tools")]
#[post("/", data = "<input>")]
async fn create_tool(
    user_id: ChatRsUserId,
    mut db: DbConnection,
    input: Json<ToolInput>,
) -> Result<Json<ChatRsTool>, ApiError> {
    let tool = ToolDbService::new(&mut db)
        .create(NewChatRsTool {
            user_id: &user_id,
            name: &input.name,
            description: &input.description,
            url: &input.url,
            method: &input.method,
            query: input.query.as_ref(),
            body: input.body.as_ref(),
        })
        .await?;

    Ok(Json(tool))
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
