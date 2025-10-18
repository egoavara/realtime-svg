# Multi-platform Dockerfile for realtime-svg backend
# Supports: linux/amd64, linux/arm64, linux/arm/v7
#
# Build arguments:
#   TARGETPLATFORM: Target platform (auto-provided by Docker Buildx)
#   TARGET_BINARY:  Binary to build (default: backend)
#   RUST_VERSION:   Rust compiler version (default: 1.86)
#
# Usage:
#   docker buildx build --platform linux/amd64,linux/arm64,linux/arm/v7 \
#     --build-arg TARGET_BINARY=backend -t myimage:latest .

# ============================================================================
# Stage 1: Planner - Generate dependency recipe
# Purpose: Analyzes Cargo.toml and generates a dependency graph (recipe.json)
#          This enables Docker layer caching for dependencies
# ============================================================================
FROM rust:1.86-bookworm AS planner
WORKDIR /app

# Install cargo-chef (cached layer)
RUN cargo install cargo-chef --version 0.1.67 --locked

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ============================================================================
# Stage 2: Builder - Cross-compilation setup and build
# Purpose: Sets up cross-compilation environment and builds the Rust binary
#          for the target platform (amd64, arm64, or armv7)
# ============================================================================
FROM --platform=$BUILDPLATFORM rust:1.86-bookworm AS builder

# Install cargo-chef (cached layer)
RUN cargo install cargo-chef --version 0.1.67 --locked

# Build arguments
ARG TARGETPLATFORM
ARG RUST_VERSION=1.86

WORKDIR /app

# Platform to Rust target triple mapping
# Maps Docker platform names to Rust target triples
RUN case "$TARGETPLATFORM" in \
      "linux/amd64") echo x86_64-unknown-linux-gnu > /rust_target.txt ;; \
      "linux/arm64") echo aarch64-unknown-linux-gnu > /rust_target.txt ;; \
      "linux/arm/v7") echo armv7-unknown-linux-gnueabihf > /rust_target.txt ;; \
      *) echo "ERROR: Unsupported platform $TARGETPLATFORM" >&2 && exit 1 ;; \
    esac

# Add Rust target for cross-compilation
RUN export RUST_TARGET=$(cat /rust_target.txt) && \
    echo "Adding Rust target: $RUST_TARGET" && \
    rustup target add $RUST_TARGET

# Install cross-compilation toolchains and C libraries
# Required for linking Rust binaries on different architectures
RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc-aarch64-linux-gnu \
    gcc-arm-linux-gnueabihf \
    gcc-x86-64-linux-gnu \
    libc6-dev-arm64-cross \
    libc6-dev-armhf-cross \
    libc6-dev-amd64-cross \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency recipe from planner stage
COPY --from=planner /app/recipe.json recipe.json

# Cook dependencies (cached layer)
# This layer is cached unless Cargo.toml changes, saving 5-10 minutes on rebuilds
RUN export RUST_TARGET=$(cat /rust_target.txt) && \
    case "$RUST_TARGET" in \
      "aarch64-unknown-linux-gnu") \
        export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc ;; \
      "armv7-unknown-linux-gnueabihf") \
        export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc ;; \
    esac && \
    cargo chef cook --release --target $RUST_TARGET --recipe-path recipe.json

# Copy source code (after dependency cooking to preserve cache)
COPY . .

# Build application binary
ARG TARGET_BINARY=backend

RUN export RUST_TARGET=$(cat /rust_target.txt) && \
    case "$RUST_TARGET" in \
      "aarch64-unknown-linux-gnu") \
        export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc ;; \
      "armv7-unknown-linux-gnueabihf") \
        export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc ;; \
    esac && \
    echo "Building $TARGET_BINARY for target: $RUST_TARGET" && \
    cargo build --release --target $RUST_TARGET --bin $TARGET_BINARY && \
    mv target/$RUST_TARGET/release/$TARGET_BINARY /app/$TARGET_BINARY

# ============================================================================
# Stage 4: Runtime - Minimal runtime image
# Purpose: Minimal Debian-based image with only the binary and runtime deps
#          Final image size: ~80-100MB (vs 2GB+ build image)
# ============================================================================
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
# - ca-certificates: TLS/HTTPS support (required for Redis TLS, external APIs)
# - tini: Proper PID 1 init system for signal handling and zombie reaping
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    tini \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder stage
ARG TARGET_BINARY=backend
COPY --from=builder /app/$TARGET_BINARY /usr/local/bin/$TARGET_BINARY

# Set executable permissions
RUN chmod +x /usr/local/bin/$TARGET_BINARY

# OCI image labels for metadata
# See: https://github.com/opencontainers/image-spec/blob/main/annotations.md
LABEL org.opencontainers.image.title="realtime-svg-backend" \
      org.opencontainers.image.description="Multiplatform Rust backend for realtime-svg (SVG streaming service)" \
      org.opencontainers.image.authors="realtime-svg team" \
      org.opencontainers.image.source="https://github.com/egoavara/realtime-svg" \
      org.opencontainers.image.licenses="MIT" \
      org.opencontainers.image.version="0.1.0"

# Use tini as PID 1 for proper signal handling (SIGTERM, SIGINT)
# Tini ensures signals are properly forwarded to the application
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/usr/local/bin/backend"]
