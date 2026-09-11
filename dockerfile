# =========================
# Build stage
# =========================
FROM rust:bookworm AS builder

WORKDIR /app

COPY . .

RUN cargo build --release


# =========================
# Runtime stage
# =========================
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/aurawear /app/aurawear

COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/public/css /app/public/css
COPY --from=builder /app/public/js /app/public/js

RUN mkdir -p /app/public/uploads

EXPOSE 3000

CMD ["/app/aurawear"]