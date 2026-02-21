# ---- Build stage ----
FROM rust:1.75-slim AS builder

WORKDIR /app

# Cache dependency compilation separately from source
COPY Cargo.toml Cargo.lock ./
COPY packages/core/Cargo.toml packages/core/Cargo.toml

# Create a stub main so cargo can compile dependencies
RUN mkdir -p packages/core/src && echo 'fn main() {}' > packages/core/src/main.rs
RUN cargo build --release --package core 2>/dev/null; true

# Build the real binary
COPY packages/core/src packages/core/src
RUN touch packages/core/src/main.rs && cargo build --release --package core

# ---- Runtime stage ----
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/core /usr/local/bin/stellar-fee-tracker

EXPOSE 8080

CMD ["stellar-fee-tracker"]
