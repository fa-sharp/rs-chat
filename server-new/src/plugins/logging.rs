use std::str::FromStr;

use anyhow::Context;
use axum::{extract::Request, http::HeaderName};
use tower::ServiceBuilder;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

use crate::plugins::AxumPlugin;

pub fn plugin() -> AxumPlugin {
    AxumPlugin::named("Request logs").on_setup(|app, router| {
        const LOG_LEVEL: Level = Level::INFO;
        let request_id_header = HeaderName::from_str(&app.config().server.request_id_header)
            .context("invalid request ID header")?;

        let trace_layer = TraceLayer::new_for_http()
            .make_span_with({
                let id_header = request_id_header.clone();
                move |req: &Request| {
                    tracing::span!(LOG_LEVEL, "request",
                        method = %req.method(),
                        uri = %req.uri(),
                        id = req.headers().get(&id_header).and_then(|id| id.to_str().ok()),
                    )
                }
            })
            .on_request(DefaultOnRequest::new().level(LOG_LEVEL))
            .on_response(DefaultOnResponse::new().level(LOG_LEVEL));

        let logging_service = ServiceBuilder::new()
            .layer(SetRequestIdLayer::new(
                request_id_header.clone(),
                MakeRequestUuid,
            ))
            .layer(trace_layer)
            .layer(PropagateRequestIdLayer::new(request_id_header));

        Ok(router.layer(logging_service))
    })
}
