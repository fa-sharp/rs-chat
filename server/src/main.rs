use chat_rs_api::build_rocket;
use rocket::launch;

#[launch]
pub fn rocket() -> _ {
    setup_logging();
    build_rocket()
}

/// Setup tracing and logging (JSON logs in release mode)
fn setup_logging() {
    if cfg!(not(debug_assertions)) {
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
}
