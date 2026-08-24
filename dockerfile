# Stage 1: Build the Rust Binary
FROM rust:1.75-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Stage 2: Run the Application
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/aurawear /app/aurawear
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/public /app/public

EXPOSE 3000
CMD ["./aurawear"]