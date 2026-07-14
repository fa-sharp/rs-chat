use std::time::Duration;

use axum::http::StatusCode;
use tower::ServiceBuilder;
use tower_http::{limit::RequestBodyLimitLayer, timeout::TimeoutLayer};

use crate::plugins::AxumPlugin;

/// # Security plugin
/// Includes body limiter, request timeout, and security headers.
pub fn plugin() -> AxumPlugin {
    AxumPlugin::named("Security").on_setup(|app, router| {
        let security_headers = axum_helmet::Helmet::new()
            .add(axum_helmet::CrossOriginOpenerPolicy::same_origin())
            .add(axum_helmet::CrossOriginResourcePolicy::same_origin())
            .add(axum_helmet::ReferrerPolicy::no_referrer())
            .add(axum_helmet::XContentTypeOptions::nosniff())
            .add(axum_helmet::XFrameOptions::same_origin())
            .into_layer()?;

        let service = ServiceBuilder::new()
            .layer(RequestBodyLimitLayer::new(app.config().security.body_limit))
            .layer(TimeoutLayer::with_status_code(
                StatusCode::REQUEST_TIMEOUT,
                Duration::from_secs(app.config().security.request_timeout),
            ))
            .layer(security_headers);

        Ok(router.layer(service))
    })
}
