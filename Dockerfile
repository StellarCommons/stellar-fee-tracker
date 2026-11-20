# Multi-stage build
FROM rust:1.99.1-slim AS builder
# native-tls (sqlx, reqwest) needs OpenSSL headers and pkg-config
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# Dependency layer: manifests plus stub sources, so source edits do not rebuild dependencies.
COPY Cargo.toml Cargo.lock ./
COPY packages/core/Cargo.toml packages/core/Cargo.toml
COPY packages/devkit/Cargo.toml packages/devkit/Cargo.toml
RUN mkdir -p packages/core/src packages/devkit/src \
    && echo 'fn main() {}' | tee packages/core/src/main.rs packages/devkit/src/main.rs > /dev/null \
    && touch packages/core/src/lib.rs packages/devkit/src/lib.rs \
    && cargo build --release -p stellar-fee-tracker

# Real sources; build only the core package (devkit is not needed in the image).
COPY packages/core packages/core
RUN touch packages/core/src/main.rs packages/core/src/lib.rs \
    && cargo build --release -p stellar-fee-tracker

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl libssl3 && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /data
COPY --from=builder /app/target/release/stellar-fee-tracker /usr/local/bin/
ENV DATABASE_URL=sqlite:///data/stellar_fees.db
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
    CMD curl -f "http://localhost:${PORT:-${API_PORT:-8080}}/health" || exit 1
CMD ["stellar-fee-tracker"]
