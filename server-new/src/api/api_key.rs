use aide::{OperationInput, OperationIo, axum::ApiRouter};
use axum::{Json, extract::State, http::StatusCode};
use axum_typed_routing::{TypedApiRouter, api_route};
use derive_more::Deref;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    api::ApiTag,
    db::models::ChatRsApiKey,
    error::AppResult,
    extractors::{CurrentUser, Database},
    state::AppState,
};

pub fn routes() -> ApiRouter<AppState> {
    ApiRouter::new()
        .typed_api_route(list_api_keys)
        .typed_api_route(create_api_key)
        .typed_api_route(delete_api_key)
}

#[api_route(GET "/" with AppState {
    summary: "List API keys",
    transform: |op| op.tag(ApiTag::ApiKey.into()),
})]
async fn list_api_keys(
    CurrentUser { user_id }: CurrentUser,
    Database(mut db): Database,
) -> AppResult<Json<Vec<ChatRsApiKey>>> {
    let keys = db.api_keys().find_by_user_id(&user_id).await?;
    Ok(Json(keys))
}

#[derive(Deserialize, JsonSchema)]
struct ApiKeyCreateInput {
    name: String,
}

#[derive(Serialize, JsonSchema)]
struct ApiKeyCreateResponse {
    id: Uuid,
    key: String,
}

#[api_route(POST "/" {
    summary: "Create API key",
    transform: |op| op.tag(ApiTag::ApiKey.into()),
})]
async fn create_api_key(
    CurrentUser { user_id }: CurrentUser,
    Database(mut db): Database,
    State(state): State<AppState>,
    input: Json<ApiKeyCreateInput>,
) -> AppResult<Json<ApiKeyCreateResponse>> {
    let (id, key) = state
        .auth_service()
        .api_keys()
        .create_api_key(&mut db, &user_id, &input.name)
        .await?;

    Ok(Json(ApiKeyCreateResponse { id, key }))
}

#[derive(Deref, Deserialize, OperationIo, JsonSchema)]
pub struct ApiKeyPath {
    pub id: Uuid,
}

#[derive(Deref, Serialize, Deserialize, JsonSchema)]
pub struct UuidPath(pub Uuid);
impl OperationInput for UuidPath {
    fn operation_input(
        ctx: &mut aide::generate::GenContext,
        operation: &mut aide::openapi::Operation,
    ) {
        use aide::openapi::{
            Parameter, ParameterData, ParameterSchemaOrContent, ReferenceOr, SchemaObject,
        };

        operation
            .parameters
            .push(ReferenceOr::Item(Parameter::Path {
                parameter_data: ParameterData {
                    name: UuidPath::schema_name().into(),
                    description: None,
                    required: true,
                    deprecated: Default::default(),
                    format: ParameterSchemaOrContent::Schema(SchemaObject {
                        json_schema: UuidPath::json_schema(&mut ctx.schema),
                        external_docs: None,
                        example: None,
                    }),
                    example: Default::default(),
                    examples: Default::default(),
                    explode: Default::default(),
                    extensions: Default::default(),
                },
                style: aide::openapi::PathStyle::Simple,
            }))
    }
}

#[api_route(DELETE "/{id}" with AppState {
    summary: "Delete API key",
    responses: { 204: () },
    transform: |op| op.tag(ApiTag::ApiKey.into()),
})]
async fn delete_api_key(
    id: UuidPath,
    CurrentUser { user_id }: CurrentUser,
    Database(mut db): Database,
) -> AppResult<StatusCode> {
    let _ = db.api_keys().delete(&user_id, &id).await?;

    Ok(StatusCode::NO_CONTENT)
}
