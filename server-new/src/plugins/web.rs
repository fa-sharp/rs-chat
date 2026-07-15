use axum::{
    Router,
    http::{HeaderValue, header},
    middleware,
    response::Response,
};
use tower_http::services::{ServeDir, ServeFile};

use crate::plugins::AxumPlugin;

/// Adds the website / static files to the router
pub fn plugin() -> AxumPlugin {
    AxumPlugin::named("Web").on_setup(|app, router| {
        let web_files_root = &app.config().server.web_root;
        let web_service = ServeDir::new(web_files_root)
            .fallback(ServeFile::new(format!("{web_files_root}/index.html")));
        let web_router = Router::new()
            .fallback_service(web_service)
            .layer(middleware::from_fn(cache_immutable_assets));

        Ok(router.merge(web_router))
    })
}

/// Set cache headers for the website's immutable assets at `/assets/*`
async fn cache_immutable_assets(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    let is_immutable_asset = req.uri().path().starts_with("/assets/");

    let mut response = next.run(req).await;
    if response.status().is_success() && is_immutable_asset {
        response.headers_mut().insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=31536000, immutable"),
        );
    }

    response
}
