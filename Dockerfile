# ─────────────────────────────────────────
# Stage 1: builder
# ─────────────────────────────────────────
FROM rust:1.88-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    libpq-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache dependency layer
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && \
    cargo build --release 2>&1 | tail -5 || true && \
    rm -rf src

# Build application
COPY . .
RUN touch src/main.rs && \
    cargo build --release --bin rust-expense-api

# ─────────────────────────────────────────
# Stage 2: runtime
# ─────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/rust-expense-api ./rust-expense-api
RUN mkdir -p /app/uploads

EXPOSE 8080
CMD ["./rust-expense-api"]
