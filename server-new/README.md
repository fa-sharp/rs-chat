# Axum Web Service Template

A production-ready template for building web services with Rust and Axum.

## Features

- **Axum** - Fast and ergonomic web framework
- **Configuration Management** - Environment-based config with `figment`
- **Structured API Errors** - JSON error responses with an `AppError` type for route handlers
- **Structured Logging** - JSON logging in production with `tracing`
- **Secure Defaults** - Default HTTP security headers, request body limit and timeout with `tower-http`
- **Optional Request Logging** - Request IDs and HTTP request/response logs with `tower-http`
- **Graceful Shutdown** - Handles SIGTERM and SIGINT signals
- ️**Plugin Architecture** - Modular app initialization with `axum-plugin`
- **Optional OpenAPI** - API documentation with `aide` (optional)
- **Docker / OCI** - Dockerfile with sensible defaults for quick deployment

## Usage

### Using cargo-generate

Install cargo-generate if you haven't already:

```bash
cargo install cargo-generate
```

Generate a new project from this template:

```bash
cargo generate --git https://git.fasharp.io/fa-sharp/axum-template
```

You'll be prompted for:
- **Project name**: The name of your new project
- **Project description**: A brief description
- **Environment variable prefix**: Prefix for env vars (e.g., `APP` for `APP_HOST`, `APP_PORT`)
- **Default port**: The server's default port
- **Default log level**: trace, debug, info, warn, or error
- **Include request logging**: Whether to include request ID and request/response logging middleware
- **Include aide**: Whether to include OpenAPI documentation support

## Configuration

Configuration is loaded from environment variables and validated in the `config.rs` file. The variable prefix is configurable during template generation.

Example with `APP` prefix:

```bash
# Required
APP_API_KEY=your-secret-key

# Optional (defaults shown)
APP_HOST=127.0.0.1
APP_PORT=8080
APP_LOG_LEVEL=info
APP_REQUEST_ID_HEADER=x-request-id
```

In development, you can use the `.env` file to set environment variables.

## Project Structure

```
.
├── src/
│   ├── routes/       # API routes
│   ├── plugins/      # Axum plugins
│   ├── config.rs     # Configuration management
│   ├── error.rs      # Structured API error handling
│   ├── lib.rs        # Axum server setup
│   ├── main.rs       # Entry point
│   └── state.rs      # Axum server state
├── Cargo.toml        # Dependencies
├── .env              # Local environment variables
└── .env.example      # Example environment variables
```

## Development

```bash
# Run in development mode (loads .env file)
cargo run

# Run with custom log level
APP_LOG_LEVEL=debug cargo run

# Build for production
cargo build --release
```

## Adding Routes

This template uses `axum-plugin` for modular initialization. To add routes:

1. Create a new plugin in a separate module
2. Register it in `lib.rs`:

```rust
pub async fn create_app() -> anyhow::Result<InitializedApp<AppState>> {
    let app = App::new()
        .register(config::plugin())
        .register(your_routes::plugin())  // Add your plugin here
        .init()
        .await?;
    
    Ok(app)
}
```

## Middleware Plugins

The template includes a `security` plugin by default. It adds common response headers, as well as a request body limiter and timeout using `tower::ServiceBuilder` and `tower-http`.

When request logging is enabled during generation, the template also includes a `logging` plugin that adds request IDs and request/response tracing.

## Error Handling

Route handlers can return `AppResult<T>`, which is an alias for `Result<T, AppError>`. `AppError` implements `IntoResponse`, so API failures are returned as JSON. It also implements `From<anyhow::Error>`, so handlers can use `?` with `anyhow` errors:

```rust
use anyhow::Context;

use crate::error::AppResult;

async fn handler() -> AppResult<String> {
    do_work().await.context("failed to do work")?;
    Ok("done".to_string())
}
```

## License

Configure your license as needed.
