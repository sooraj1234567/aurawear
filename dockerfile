# Stage 1: Build the Rust Binary
FROM rust:1.75-slim AS builder
WORKDIR /app

# Install system dependencies required for compilation
RUN apt-get update && apt-get install -y pkg-config libssl-dev build-essential

COPY . .
RUN cargo build --release

# Stage 2: Run the Application
FROM debian:bookworm-slim
WORKDIR /app

# Install runtime SSL libraries and certificates
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/aurawear /app/aurawear
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/public /app/public

EXPOSE 3000
CMD ["./aurawear"]