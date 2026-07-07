//! Application state

use std::{ops::Deref, sync::Arc};

use axum_plugin::{AppState, TypeMap};

use crate::{
    config::AppConfig,
    db::DbPool,
    services::{
        auth::{AuthService, encryption::Encryptor, oauth::OAuthProviderMap},
        chat::ChatService,
        model::ModelService,
        provider::ProviderService,
        stream::tinistream::TinistreamClient,
    },
};

/// App state stored in the Axum router
#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);

#[derive(AppState)]
pub struct AppStateInner {
    pub config: AppConfig,
    pub db_pool: DbPool,
    pub encryptor: Encryptor,
    pub http_client: reqwest::Client,
    pub oauth_providers: OAuthProviderMap,
    pub redis: fred::prelude::Pool,
    pub tinistream: TinistreamClient,
}

impl AppState {
    pub fn auth_service(&self) -> AuthService<'_> {
        AuthService::new(&self.config, &self.encryptor, &self.oauth_providers)
    }
    pub fn chat_service(&self) -> ChatService<'_> {
        ChatService::new(&self.db_pool, &self.tinistream)
    }
    pub fn provider_service(&self) -> ProviderService<'_> {
        ProviderService::new(&self.encryptor, &self.http_client)
    }
    pub fn model_service(&self) -> ModelService<'_> {
        ModelService::new(&self.redis, &self.http_client)
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
