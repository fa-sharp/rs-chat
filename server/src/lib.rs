use rocket::{fairing::AdHoc, get};
use rocket_okapi::{
    mount_endpoints_and_merged_docs, openapi, openapi_get_routes_spec,
    rapidoc::{make_rapidoc, GeneralConfig, RapiDocConfig},
    settings::{OpenApiSettings, UrlObject},
};

use crate::{
    config::{get_config_provider, AppConfig},
    db::setup_db,
};

pub mod config;
pub mod db;

/// Build the rocket server, load configuration and routes, prepare for launch
pub fn build_rocket() -> rocket::Rocket<rocket::Build> {
    let mut server = rocket::custom(get_config_provider())
        .attach(AdHoc::config::<AppConfig>())
        .attach(setup_db())
        .mount("/api/docs", get_doc_routes());

    let openapi_settings = OpenApiSettings::default();
    mount_endpoints_and_merged_docs! {
        server, "/api", openapi_settings,
        "/" => openapi_get_routes_spec![health]
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
    make_rapidoc(&RapiDocConfig {
        general: GeneralConfig {
            spec_urls: vec![UrlObject::new("OpenAPI Schema", "/api/openapi.json")],
            ..Default::default()
        },
        ..Default::default()
    })
}
