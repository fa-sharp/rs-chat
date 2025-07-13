pub mod api;
pub mod auth;
pub mod config;
pub mod db;
pub mod errors;
pub mod provider;
pub mod redis;
pub mod tools;
pub mod utils;
pub mod web;

use rocket::{fairing::AdHoc, get};
use rocket_okapi::{mount_endpoints_and_merged_docs, openapi, openapi_get_routes_spec};

use crate::{
    api::auth_undocumented_routes,
    auth::setup_auth,
    config::{get_config_provider, AppConfig},
    db::setup_db,
    errors::get_catchers,
    redis::setup_redis,
    utils::encryption::setup_encryption,
    web::setup_static_files,
};

/// Build the rocket server, load configuration and routes, prepare for launch
pub fn build_rocket() -> rocket::Rocket<rocket::Build> {
    let mut server = rocket::custom(get_config_provider())
        .attach(AdHoc::config::<AppConfig>())
        .attach(setup_db())
        .attach(setup_redis())
        .attach(setup_encryption())
        .attach(setup_auth("/api/auth"))
        .attach(setup_static_files())
        .manage(reqwest::Client::new())
        .register("/", get_catchers())
        .mount("/api/auth", auth_undocumented_routes())
        .mount("/api/docs", get_doc_routes());

    let openapi_settings = rocket_okapi::settings::OpenApiSettings::default();
    mount_endpoints_and_merged_docs! {
        server, "/api", openapi_settings,
        "/" => openapi_get_routes_spec![health],
        "/auth" => api::auth_routes(&openapi_settings),
        "/session" => api::session_routes(&openapi_settings),
        "/chat" => api::chat_routes(&openapi_settings),
        "/tool" => api::tool_routes(&openapi_settings),
        "/provider_key" => api::provider_key_routes(&openapi_settings),
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
        title: Some(String::from("RsChat API Documentation")),
        general: GeneralConfig {
            heading_text: String::from("RsChat API"),
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
