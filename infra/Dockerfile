# Fetchium API — multi-stage build
FROM rust:1.75-slim-bookworm AS builder

WORKDIR /build

# Install system deps for rusqlite (bundled) and ring (crypto)
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev clang && \
    rm -rf /var/lib/apt/lists/*

# Cache dependencies by copying manifests first
COPY Cargo.toml Cargo.lock ./
COPY crates/fetchium-core/Cargo.toml crates/fetchium-core/
COPY crates/fetchium-api/Cargo.toml  crates/fetchium-api/
COPY crates/fetchium-cli/Cargo.toml  crates/fetchium-cli/
COPY crates/fetchium-mcp/Cargo.toml  crates/fetchium-mcp/

# Stub source files for dependency caching
RUN mkdir -p crates/fetchium-core/src crates/fetchium-api/src crates/fetchium-cli/src crates/fetchium-mcp/src && \
    echo "pub fn main(){}" > crates/fetchium-cli/src/main.rs && \
    touch crates/fetchium-core/src/lib.rs crates/fetchium-api/src/lib.rs crates/fetchium-mcp/src/lib.rs

RUN cargo build --release -p fetchium-cli 2>/dev/null || true

# Build the real binary
COPY . .
RUN cargo build --release -p fetchium-cli

# ─── Runtime image ───────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/fetchium /usr/local/bin/fetchium

# Data directory for auth.db and cache
RUN mkdir -p /data
VOLUME ["/data"]

EXPOSE 3050
ENV FETCHIUM_DATA_DIR=/data

CMD ["fetchium", "serve", "--mode", "rest"]
