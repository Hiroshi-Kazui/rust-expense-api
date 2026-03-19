# ─────────────────────────────────────────
# Stage 1: builder
# ─────────────────────────────────────────
FROM rust:1.88-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    libpq-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache dependency layer (workspace-aware)
COPY Cargo.toml Cargo.lock* ./
COPY frontend/Cargo.toml ./frontend/Cargo.toml
RUN mkdir -p src frontend/src && \
    echo 'fn main() {}' > src/main.rs && \
    echo 'fn main() {}' > frontend/src/main.rs && \
    cargo build --release --bin rust-expense-api 2>&1 | tail -5 || true && \
    rm -rf src frontend/src

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
