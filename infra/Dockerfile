# HyperSearchX API — multi-stage build
FROM rust:1.75-slim-bookworm AS builder

WORKDIR /build

# Install system deps for rusqlite (bundled) and ring (crypto)
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev clang && \
    rm -rf /var/lib/apt/lists/*

# Cache dependencies by copying manifests first
COPY Cargo.toml Cargo.lock ./
COPY crates/hsx-core/Cargo.toml crates/hsx-core/
COPY crates/hsx-api/Cargo.toml  crates/hsx-api/
COPY crates/hsx-cli/Cargo.toml  crates/hsx-cli/
COPY crates/hsx-mcp/Cargo.toml  crates/hsx-mcp/

# Stub source files for dependency caching
RUN mkdir -p crates/hsx-core/src crates/hsx-api/src crates/hsx-cli/src crates/hsx-mcp/src && \
    echo "pub fn main(){}" > crates/hsx-cli/src/main.rs && \
    touch crates/hsx-core/src/lib.rs crates/hsx-api/src/lib.rs crates/hsx-mcp/src/lib.rs

RUN cargo build --release -p hsx-cli 2>/dev/null || true

# Build the real binary
COPY . .
RUN cargo build --release -p hsx-cli

# ─── Runtime image ───────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/hsx /usr/local/bin/hsx

# Data directory for auth.db and cache
RUN mkdir -p /data
VOLUME ["/data"]

EXPOSE 3000
ENV HSX_DATA_DIR=/data

CMD ["hsx", "serve", "--mode", "rest"]
