use rocket::{
    fairing::AdHoc,
    figment::{
        providers::{Env, Format, Toml},
        Figment,
    },
};
use serde::Deserialize;

use crate::utils::s3_storage::S3Storage;

/// S3 storage configuration
#[derive(Debug, Deserialize)]
pub struct S3Config {
    pub s3_secret_key: String,
    pub s3_access_key: String,
    pub s3_endpoint_url: String,
    pub s3_region: Option<String>,
    pub s3_bucket: Option<String>,
}

/// Setup S3 storage service if relevant environment variables are present
pub fn setup_s3_storage() -> AdHoc {
    AdHoc::on_ignite("S3 Storage", |rocket| async {
        let Ok(config) = get_config_provider().extract::<S3Config>() else {
            rocket::info!("S3 Storage: configuration not found - skipping setup");
            return rocket.manage::<Option<S3Storage>>(None);
        };

        let region = config.s3_region.unwrap_or(String::from("us-east-1"));
        let bucket_name = config.s3_bucket.unwrap_or(String::from("rs-chat"));
        let s3_service = S3Storage::new(
            &config.s3_endpoint_url,
            &config.s3_access_key,
            &config.s3_secret_key,
            region,
            bucket_name.clone(),
        );
        match s3_service.check_bucket().await {
            Ok(true) => rocket::info!("S3 Storage: bucket '{}' found", &bucket_name),
            Ok(false) => rocket::warn!("S3 Storage: bucket '{}' not found", &bucket_name),
            Err(err) => rocket::error!(
                "S3 Storage: error connecting to bucket '{}': {:?} {:?}",
                &bucket_name,
                err.meta().code(),
                err.meta().message()
            ),
        }

        rocket.manage(Some(s3_service))
    })
}

/// Builds and returns a Figment configuration provider that merges variables from:
/// 1. Rocket.toml file
/// 1. Environment variables prefixed with `RS_CHAT_`. In debug/dev mode, will also load
/// variables from local `.env` file
fn get_config_provider() -> Figment {
    #[cfg(debug_assertions)]
    if let Err(e) = dotenvy::dotenv() {
        println!("Failed to read .env file: {}", e);
    }

    Figment::new()
        .merge(Toml::file("Rocket.toml").nested())
        .merge(Env::prefixed("RS_CHAT_").global())
}
