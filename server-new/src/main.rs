use std::net::SocketAddr;

use rs_chat_api::create_app;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Read .env file in debug mode
    #[cfg(debug_assertions)]
    dotenvy::dotenv().ok();

    // Initialize logging
    let (log_filter_handle, _log_guard) = init_logging();

    // Build server
    let app = create_app().await?;
    let config = &app.state().config;

    // Set log level from config
    let env_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .parse(&config.server.log_level)?;
    log_filter_handle.reload(env_filter)?;

    // Start listening for requests
    let addr = SocketAddr::new(config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Server listening on http://{}...", listener.local_addr()?);
    axum::serve(
        listener,
        app.router()
            .into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal(app.shutdown()))
    .await?;

    Ok(())
}

fn init_logging() -> (
    tracing_subscriber::reload::Handle<EnvFilter, Registry>,
    tracing_appender::non_blocking::WorkerGuard,
) {
    let init_log_level = std::env::var("RS_CHAT_SERVER__LOG_LEVEL")
        .or_else(|_| std::env::var("RS_CHAT_LOG_LEVEL"))
        .unwrap_or("info".into());
    let (writer, guard) = tracing_appender::non_blocking(std::io::stdout());
    let (filter_layer, filter_handle) =
        tracing_subscriber::reload::Layer::new(EnvFilter::new(init_log_level));

    if cfg!(debug_assertions) {
        tracing_subscriber::registry()
            .with(filter_layer)
            .with(tracing_subscriber::fmt::layer().with_writer(writer))
            .init();
    } else {
        let json_layer = tracing_subscriber::fmt::layer()
            .json()
            .flatten_event(true)
            .with_current_span(false)
            .with_writer(writer);

        tracing_subscriber::registry()
            .with(filter_layer)
            .with(json_layer)
            .init();
    }

    (filter_handle, guard)
}

/// Shutdown signal: listens for Ctrl-C, SIGINT, SIGTERM signals
async fn shutdown_signal(on_shutdown: impl Future + Send) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to register Ctrl-C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to register SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("Received shutdown signal, shutting down server...");
    on_shutdown.await;
}
