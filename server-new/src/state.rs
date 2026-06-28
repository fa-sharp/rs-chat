//! Application state

use std::{ops::Deref, sync::Arc};

use axum_plugin::{AppState, TypeMap};

use crate::{
    config::AppConfig,
    db::DbPool,
    services::auth::{AuthService, oauth::OAuthProviderMap},
};

/// App state stored in the Axum router
#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);

#[derive(AppState)]
pub struct AppStateInner {
    pub config: AppConfig,
    pub http_client: reqwest::Client,
    pub db_pool: DbPool,
    pub redis: fred::prelude::Pool,
    pub oauth_providers: OAuthProviderMap,
}

impl AppState {
    pub fn auth_service(&self) -> AuthService<'_> {
        AuthService::new(
            &self.db_pool,
            &self.config,
            &self.http_client,
            &self.oauth_providers,
        )
    }
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<TypeMap> for AppState {
    type Error = anyhow::Error;

    fn try_from(map: TypeMap) -> Result<Self, Self::Error> {
        Ok(Self(Arc::new(AppStateInner::try_from(map)?)))
    }
}
