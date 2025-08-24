use rocket::{get, serde::json::Json, Route, State};
use rocket_okapi::{
    okapi::openapi3::OpenApi, openapi, openapi_get_routes_spec, settings::OpenApiSettings,
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{auth::ChatRsUserId, config::AppConfig, errors::ApiError, redis::ExclusiveClientPool};

pub fn get_routes(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        settings: get_info
    ]
}

#[derive(Debug, Serialize, JsonSchema)]
struct InfoResponse {
    version: String,
    url: String,
    redis: RedisStats,
}

#[derive(Debug, Serialize, JsonSchema)]
struct RedisStats {
    /// Number of static connections
    r#static: usize,
    /// Number of current streaming connections
    streaming: usize,
    /// Number of available streaming connections
    streaming_available: usize,
    /// Maximum number of streaming connections
    streaming_max: usize,
}

/// # Get info
/// Get information about the server
#[openapi]
#[get("/")]
async fn get_info(
    _user_id: ChatRsUserId,
    app_config: &State<AppConfig>,
    redis_pool: &State<ExclusiveClientPool>,
) -> Result<Json<InfoResponse>, ApiError> {
    let redis_status = redis_pool.status();
    let redis_stats = RedisStats {
        r#static: app_config.redis_pool.unwrap_or(4),
        streaming: redis_status.size,
        streaming_max: redis_status.max_size,
        streaming_available: redis_status.available,
    };

    Ok(Json(InfoResponse {
        version: format!("v{}", env!("CARGO_PKG_VERSION")),
        url: app_config.server_address.clone(),
        redis: redis_stats,
    }))
}
