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

RUN apt-get update -qq && \
    apt-get install ca-certificates -qq -y && \
    apt-get clean

# Copy server binary
COPY --from=build --chown=appuser /app/run-server /usr/local/bin/

# Run server
WORKDIR /app
ENV RS_CHAT_SERVER__HOST=0.0.0.0
CMD ["run-server"]
