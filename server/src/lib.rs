pub mod api;
pub mod auth;
pub mod config;
pub mod db;
pub mod errors;
pub mod provider;
pub mod redis;
pub mod utils;
pub mod web;

use rocket::{fairing::AdHoc, get};
use rocket_okapi::{mount_endpoints_and_merged_docs, openapi, openapi_get_routes_spec};

use crate::{
    auth::{setup_encryption, setup_oauth, setup_session},
    config::{get_config_provider, AppConfig},
    db::setup_db,
    errors::get_catchers,
    redis::setup_redis,
    web::setup_static_files,
};

/// Build the rocket server, load configuration and routes, prepare for launch
pub fn build_rocket() -> rocket::Rocket<rocket::Build> {
    let mut server = rocket::custom(get_config_provider())
        .attach(AdHoc::config::<AppConfig>())
        .attach(setup_db())
        .attach(setup_redis())
        .attach(setup_encryption())
        .attach(setup_session())
        .attach(setup_oauth())
        .attach(setup_static_files())
        .register("/", get_catchers())
        .mount("/api/docs", get_doc_routes())
        .mount("/api/auth", api::oauth_routes());

    let openapi_settings = rocket_okapi::settings::OpenApiSettings::default();
    mount_endpoints_and_merged_docs! {
        server, "/api", openapi_settings,
        "/" => openapi_get_routes_spec![health],
        "/auth" => api::auth_routes(&openapi_settings),
        "/session" => api::session_routes(&openapi_settings),
        "/chat" => api::chat_routes(&openapi_settings),
        "/provider" => api::provider_routes(&openapi_settings),
        "/api_key" => api::api_key_routes(&openapi_settings),
    };

    server
}

/// Health route
#[openapi]
#[get("/health")]
async fn health() -> String {
    "OK".to_owned()
}

/// Create the OpenAPI doc routes
fn get_doc_routes() -> impl Into<Vec<rocket::Route>> {
    use rocket_okapi::{
        rapidoc::{make_rapidoc, GeneralConfig, Layout, LayoutConfig, RapiDocConfig, RenderStyle},
        settings::UrlObject,
    };

    make_rapidoc(&RapiDocConfig {
        general: GeneralConfig {
            spec_urls: vec![UrlObject::new("OpenAPI Schema", "/api/openapi.json")],
            ..Default::default()
        },
        layout: LayoutConfig {
            layout: Layout::Column,
            render_style: RenderStyle::View,
            ..Default::default()
        },
        ..Default::default()
    })
}
