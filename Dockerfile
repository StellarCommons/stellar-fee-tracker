# Multi-stage build
FROM rust:1.99.1-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl libssl3 && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /data
COPY --from=builder /app/target/release/stellar-fee-tracker /usr/local/bin/
EXPOSE 8080
CMD ["stellar-fee-tracker"]
