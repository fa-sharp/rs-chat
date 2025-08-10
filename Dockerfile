ARG NODE_VERSION=22
ARG RUST_VERSION=1.85
ARG DEBIAN_VERSION=bookworm

### Build Rust backend ###
FROM rust:${RUST_VERSION}-slim-${DEBIAN_VERSION} AS backend-build
WORKDIR /app

COPY ./server/src src
COPY ./server/migrations migrations
COPY ./server/Cargo.toml ./server/Cargo.lock ./

ARG pkg=chat-rs-api

RUN apt-get update -qq && apt-get install -y -qq pkg-config libpq-dev && apt-get clean
RUN --mount=type=cache,id=s/6916a5c5-fde9-46e7-934c-cf425dd70d0e-rust_target,target=/app/target \
    --mount=type=cache,id=s/6916a5c5-fde9-46e7-934c-cf425dd70d0e-cargo_registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=s/6916a5c5-fde9-46e7-934c-cf425dd70d0e-cargo_git,target=/usr/local/cargo/git \
    set -eux; \
    cargo build --release; \
    objcopy --compress-debug-sections target/release/$pkg ./run-server

### Build Vite frontend with pnpm ###
FROM node:${NODE_VERSION}-${DEBIAN_VERSION}-slim AS frontend-build
WORKDIR /app

RUN npm install -g pnpm
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"

COPY ./web/package.json ./web/pnpm-lock.yaml ./
RUN --mount=type=cache,id=s/6916a5c5-fde9-46e7-934c-cf425dd70d0e-pnpm,target=/pnpm/store pnpm install --frozen-lockfile

COPY ./web/src src
COPY ./web/public public
COPY ./web/index.html ./web/tsconfig.json ./web/vite.config.ts ./
RUN pnpm run build

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
COPY --from=frontend-build /app/dist /var/www
COPY --from=backend-build /app/run-server /usr/local/bin/

# Run
ENV RS_CHAT_STATIC_PATH=/var/www
ENV RS_CHAT_ADDRESS=0.0.0.0
ENV RS_CHAT_PORT=8080
EXPOSE 8080
CMD ["run-server"]
