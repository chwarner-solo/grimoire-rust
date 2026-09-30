# ── Stage 1: build ──────────────────────────────────────────────────────────
FROM rust:1.98-slim AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

RUN cargo build --release -p grimoire

# ── Stage 2: runtime ────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/grimoire /app/grimoire

# Data is mounted here at runtime
VOLUME ["/data"]

# Logging: LOG_FORMAT=json emits newline-delimited JSON for CloudWatch / Cloud Logging.
# Switch to LOG_FORMAT=pretty for local dev or when using a log forwarder that expects text.
# RUST_LOG controls verbosity — use grimoire=debug to increase detail.
ENV DATA_DIR=/data \
    PORT=3000 \
    LOG_FORMAT=json \
    RUST_LOG=grimoire=info,tower_http=info

EXPOSE 3000

ENTRYPOINT ["/app/grimoire"]
