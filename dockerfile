# ==========================================
# Stage 1: Build the Rust Application
# ==========================================
FROM rust:1.88-slim-bookworm AS builder

WORKDIR /app

# Install dependencies required to compile Rust crates
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
        build-essential \
        ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the complete project
COPY . .

# Build the application in release mode
RUN cargo build --release


# ==========================================
# Stage 2: Production Runtime
# ==========================================
FROM debian:bookworm-slim

WORKDIR /app

# Runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl3 && \
    rm -rf /var/lib/apt/lists/*

# Copy compiled application binary from builder stage
COPY --from=builder /app/target/release/aurawear /app/aurawear

# Copy application assets
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/public /app/public

# Render's default web-service port
EXPOSE 10000

# Start the application
CMD ["./aurawear"]