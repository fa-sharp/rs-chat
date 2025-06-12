ARG NODE_VERSION=20
ARG RUST_VERSION=1.84
ARG DEBIAN_VERSION=bookworm

### Build Rust backend ###
FROM rust:${RUST_VERSION}-slim-${DEBIAN_VERSION} AS backend-build
WORKDIR /app

COPY ./server/src src
COPY ./server/migrations migrations
COPY ./server/Cargo.toml ./server/Cargo.lock ./

ARG pkg=chat-rs-api

RUN apt-get update -qq && apt-get install -y -qq pkg-config libpq-dev && apt-get clean
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    set -eux; \
    cargo build --release; \
    objcopy --compress-debug-sections target/release/$pkg ./run-server

### Final image ###
FROM debian:${DEBIAN_VERSION}-slim

# Install required dependencies
RUN apt-get update -qq && apt-get install -y -qq ca-certificates && apt-get clean

# Use non-root user
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

# Copy app files
# COPY --from=frontend-build /app/build /srv
COPY --from=backend-build /app/run-server /usr/local/bin/

# Run
ENV CHAT_RS_ADDRESS=0.0.0.0
ENV CHAT_RS_PORT=8080
EXPOSE 8080
CMD ["run-server"]
