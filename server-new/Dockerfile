# Image versions (can be overridden by args when building)
ARG RUST_VERSION=1.96
ARG DEBIAN_VERSION=bookworm

### Build server ###
FROM rust:${RUST_VERSION}-slim-${DEBIAN_VERSION} AS build
WORKDIR /app

# Copy all necessary files to build the server
COPY Cargo.lock Cargo.toml ./
COPY ./crates ./crates
COPY ./migrations ./migrations
COPY ./src ./src

ARG pkg=rs-chat-api

RUN --mount=type=cache,id=rust_target,target=/app/target \
    --mount=type=cache,id=cargo_registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=cargo_git,target=/usr/local/cargo/git \
    set -eux; \
    cargo build --package $pkg --release --locked; \
    objcopy --compress-debug-sections target/release/$pkg ./run-server


### Run server ###
FROM debian:${DEBIAN_VERSION}-slim AS run

# Create non-root user
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/home/appuser" \
    --shell "/sbin/nologin" \
    --uid "${UID}" \
    appuser
USER appuser

# Copy server binary
COPY --from=build --chown=appuser /app/run-server /usr/local/bin/

# Run server
WORKDIR /app
COPY --chown=appuser config.toml ./config.toml
ENV RS_CHAT_SERVER__HOST=0.0.0.0
CMD ["run-server"]
