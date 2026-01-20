# Build stage
FROM rust:1.83-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Install nightly Rust toolchain for edition2024 support
RUN rustup toolchain install nightly && \
    rustup default nightly

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY core/Cargo.toml ./core/
COPY listeners/Cargo.toml ./listeners/

# Create dummy source files to cache dependencies
RUN mkdir -p core/src listeners/src && \
    echo "fn main() {}" > core/src/lib.rs && \
    echo "fn main() {}" > listeners/src/main.rs

# Build dependencies - this layer will be cached
RUN cargo build --release --package listeners

# Remove dummy files
RUN rm -rf core/src listeners/src

# Copy actual source code
COPY core/ ./core/
COPY listeners/ ./listeners/
COPY proto/ ./proto/

# Build the application
# Touch main.rs to force rebuild of the app with actual source
RUN touch listeners/src/main.rs && \
    cargo build --release --package listeners

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1001 -U appuser

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/listeners /app/listeners

# Change ownership to non-root user
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose port if needed (uncomment and adjust if your service uses a specific port)
# EXPOSE 8080

# Run the binary
ENTRYPOINT ["/app/listeners"]
