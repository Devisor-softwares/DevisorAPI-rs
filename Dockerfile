# Multi-stage build for Rust backend
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (cache layer)
RUN cargo build --release && rm -rf src

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 rustuser

WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/backend ./backend
COPY --from=builder /app/target/release/simple ./simple

# Copy migrations
COPY --from=builder /app/migrations ./migrations

# Change ownership
RUN chown -R rustuser:rustuser /app

USER rustuser

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://127.0.0.1:3000/health || exit 1

EXPOSE 3000

# Default to simple mode, can be overridden
CMD ["./simple"]
