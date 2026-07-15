use axum_plugin::AdHocPlugin;

use crate::{config::AppConfig, state::AppState};

pub mod auth;
pub mod clients;
pub mod database;
pub mod logging;
pub mod redis;
pub mod security;
pub mod web;

/// Shared plugin type with correct state and config type parameters
pub type AxumPlugin = AdHocPlugin<AppState, AppConfig>;
