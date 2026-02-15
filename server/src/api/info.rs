use rocket::{get, serde::json::Json, Route, State};
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
};
use schemars::JsonSchema;
use serde::Serialize;

pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        settings: get_info
    ]
}

#[derive(Debug, Serialize, JsonSchema)]
struct InfoResponse {
    version: String,
    url: String,
}

/// # Get info
/// Get information about the server
#[openapi]
#[get("/")]
async fn get_info(app_config: &State<crate::config::AppConfig>) -> Json<InfoResponse> {
    Json(InfoResponse {
        version: format!("v{}", env!("CARGO_PKG_VERSION")),
        url: app_config.server_address.clone(),
    })
}
