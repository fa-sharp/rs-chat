use std::time::Duration;

use axum::http::StatusCode;
use axum_plugin::AdHocPlugin;
use tower::ServiceBuilder;
use tower_http::{limit::RequestBodyLimitLayer, timeout::TimeoutLayer};

use crate::state::AppState;

/// # Security plugin
/// Includes body limiter, request timeout, and security headers.
pub fn plugin() -> AdHocPlugin<AppState> {
    AdHocPlugin::named("Security").on_setup(|router, state: &AppState| {
        let security_headers = axum_helmet::Helmet::new()
            .add(axum_helmet::CrossOriginOpenerPolicy::same_origin())
            .add(axum_helmet::CrossOriginResourcePolicy::same_origin())
            .add(axum_helmet::ReferrerPolicy::no_referrer())
            .add(axum_helmet::XContentTypeOptions::nosniff())
            .add(axum_helmet::XFrameOptions::same_origin())
            .into_layer()?;

        let service = ServiceBuilder::new()
            .layer(RequestBodyLimitLayer::new(state.config.body_limit))
            .layer(TimeoutLayer::with_status_code(
                StatusCode::REQUEST_TIMEOUT,
                Duration::from_secs(state.config.request_timeout),
            ))
            .layer(security_headers);

        Ok(router.layer(service))
    })
}
