mod data_guard;
mod local;

pub use data_guard::*;
pub use local::*;

use std::path::PathBuf;

use rocket::fairing::AdHoc;

use crate::config::get_app_config;

/// Default data directory path.
pub const DEFAULT_DATA_DIR: &str = "/data";

/// Setup file reading and writing for the Rocket application.
pub fn setup_storage() -> AdHoc {
    AdHoc::on_ignite("Storage", |rocket| async {
        let app_config = get_app_config(&rocket);
        let data_dir = app_config.data_dir.as_deref().unwrap_or(DEFAULT_DATA_DIR);
        let storage_path = PathBuf::from(data_dir).join("storage");
        let storage = LocalStorage::new(storage_path);

        rocket.manage(storage)
    })
}
