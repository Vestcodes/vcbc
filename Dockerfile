# VCBC Blockchain Docker Image
FROM rust:1.91-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./

# Create dummy src to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false vcbc

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/vcbc /usr/local/bin/vcbc

# Create data directory
RUN mkdir -p /app/data && chown vcbc:vcbc /app/data

# Switch to non-root user
USER vcbc

# Expose ports (HTTP API and P2P)
EXPOSE 8080 9090

# Set data directory
ENV VCBC_DATA_DIR=/app/data

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Default command
CMD ["vcbc", "--help"]
