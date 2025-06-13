/// Setup tracing and logging, using JSON logs and default WARN level
pub fn setup_json_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::WARN.into())
                .with_regex(false)
                .from_env_lossy(),
        )
        .json()
        .init();
}
