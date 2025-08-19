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
    api::secret::SecretInput,
    auth::ChatRsUserId,
    db::{
        models::{
            ChatRsExecutedToolCall, ChatRsExternalApiTool, ChatRsMessageMeta, ChatRsMessageRole,
            ChatRsSystemTool, NewChatRsExternalApiTool, NewChatRsMessage, NewChatRsSecret,
            NewChatRsSystemTool,
        },
        services::{ChatDbService, SecretDbService, ToolDbService},
        DbConnection,
    },
    errors::ApiError,
    provider::LlmToolType,
    tools::{
        ChatRsExternalApiToolConfig, ChatRsSystemToolConfig, ToolError, ToolLog, ToolResponseFormat,
    },
    utils::{Encryptor, SenderWithLogging},
};

pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings:
        get_all_tools,
        execute_tool,
        create_tool,
        delete_system_tool,
        delete_external_api_tool,
    ]
}

#[derive(JsonSchema, serde::Serialize)]
struct GetAllToolsResponse {
    /// System tools
    system: Vec<ChatRsSystemTool>,
    /// External API tools
    external_api: Vec<ChatRsExternalApiTool>,
}

/// List all tools
#[openapi(tag = "Tools")]
#[get("/")]
async fn get_all_tools(
    user_id: ChatRsUserId,
    mut db: DbConnection,
) -> Result<Json<GetAllToolsResponse>, ApiError> {
    let (system, external_api) = ToolDbService::new(&mut db).find_by_user(&user_id).await?;

    Ok(Json(GetAllToolsResponse {
        system,
        external_api,
    }))
}

#[derive(JsonSchema, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum CreateToolInput {
    /// Create a new system tool
    System(ChatRsSystemToolConfig),
    /// Create a new external API tool
    ExternalApi {
        /// The configuration for the external API tool
        #[serde(flatten)]
        config: ChatRsExternalApiToolConfig,
        /// API key / secret key
        secret_1: Option<SecretInput>,
    },
}

#[derive(JsonSchema, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum CreateToolResponse {
    System(ChatRsSystemTool),
    ExternalApi(ChatRsExternalApiTool),
}

/// Create a new tool
#[openapi(tag = "Tools")]
#[post("/", data = "<input>")]
async fn create_tool(
    user_id: ChatRsUserId,
    mut db: DbConnection,
    encryptor: &State<Encryptor>,
    input: Json<CreateToolInput>,
) -> Result<Json<CreateToolResponse>, ApiError> {
    match input.into_inner() {
        CreateToolInput::System(ref config) => {
            config.validate()?;
            let tool = ToolDbService::new(&mut db)
                .create_system_tool(NewChatRsSystemTool {
                    user_id: &user_id,
                    data: config,
                })
                .await?;
            Ok(Json(CreateToolResponse::System(tool)))
        }
        CreateToolInput::ExternalApi {
            mut config,
            secret_1,
        } => {
            config.validate()?;
            let mut secret_1_id = None;
            if let Some(secret_input) = secret_1 {
                let (ciphertext, nonce) = encryptor.encrypt_string(&secret_input.key)?;
                let new_secret_id = SecretDbService::new(&mut db)
                    .create(NewChatRsSecret {
                        user_id: &user_id,
                        name: &secret_input.name,
                        ciphertext: &ciphertext,
                        nonce: &nonce,
                    })
                    .await?;
                secret_1_id = Some(new_secret_id);
            }
            let tool = ToolDbService::new(&mut db)
                .create_external_api_tool(NewChatRsExternalApiTool {
                    user_id: &user_id,
                    data: &config,
                    secret_1: secret_1_id.as_ref(),
                })
                .await?;
            Ok(Json(CreateToolResponse::ExternalApi(tool)))
        }
    }
}

/// Execute a tool call and stream its output
#[openapi(tag = "Tools")]
#[post("/execute/<message_id>/<tool_call_id>")]
async fn execute_tool(
    user_id: ChatRsUserId,
    mut db: DbConnection,
    http_client: &State<reqwest::Client>,
    encryptor: &State<Encryptor>,
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
    let mut tool_db_service = ToolDbService::new(&mut db);
    let (system_tool, external_api_tool, secret_1) = match tool_call.tool_type {
        LlmToolType::System => {
            let tool = tool_db_service
                .find_system_tool_by_id(&user_id, &tool_call.tool_id)
                .await?
                .ok_or(ToolError::ToolNotFound)?;
            (Some(tool), None, None)
        }
        LlmToolType::ExternalApi => {
            let (tool, secret) = tool_db_service
                .find_external_api_tool_by_id(&user_id, &tool_call.tool_id)
                .await?
                .map(|(tool, secret)| {
                    let secret = secret.map(|s| encryptor.decrypt_string(&s.ciphertext, &s.nonce));
                    (tool, secret)
                })
                .ok_or(ToolError::ToolNotFound)?;
            (None, Some(tool), secret.transpose()?)
        }
    };

    let (streaming_tx, streaming_rx) = tokio::sync::mpsc::channel(50);
    let http_client = http_client.inner().clone();
    let secrets = secret_1.into_iter().collect::<Vec<_>>();

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
        let tool_result = match (system_tool, external_api_tool) {
            (Some(system_tool), None) => {
                system_tool
                    .build_executor()
                    .validate_and_execute(
                        &tool_call.tool_name,
                        &tool_call.parameters,
                        &sender_with_logging,
                    )
                    .await
            }
            (None, Some(api_tool)) => {
                api_tool
                    .build_executor()
                    .validate_and_execute(
                        &tool_call.tool_name,
                        &tool_call.parameters,
                        &secrets,
                        &http_client,
                        &sender_with_logging,
                    )
                    .await
            }
            _ => unreachable!(),
        };
        let (content, format, is_error) = match tool_result {
            Ok((response, format)) => (response, format, None),
            Err(e) => (e.to_string(), ToolResponseFormat::Text, Some(true)),
        };
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
                        tool_type: tool_call.tool_type,
                        response_format: format,
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

/// Delete a system tool
#[openapi(tag = "Tools")]
#[delete("/system/<tool_id>")]
async fn delete_system_tool(
    user_id: ChatRsUserId,
    mut db: DbConnection,
    tool_id: Uuid,
) -> Result<String, ApiError> {
    let id = ToolDbService::new(&mut db)
        .delete_system_tool(&user_id, &tool_id)
        .await?;

    Ok(id.to_string())
}

/// Delete an external API tool
#[openapi(tag = "Tools")]
#[delete("/external-api/<tool_id>")]
async fn delete_external_api_tool(
    user_id: ChatRsUserId,
    mut db: DbConnection,
    tool_id: Uuid,
) -> Result<String, ApiError> {
    let (tool, secret_1) = ToolDbService::new(&mut db)
        .find_external_api_tool_by_id(&user_id, &tool_id)
        .await?
        .ok_or(ToolError::ToolNotFound)?;
    if let Some(secret) = secret_1 {
        let _ = SecretDbService::new(&mut db)
            .delete(&user_id, &secret.id)
            .await?;
    }
    let id = ToolDbService::new(&mut db)
        .delete_external_api_tool(&user_id, &tool.id)
        .await?;

    Ok(id.to_string())
}
