# ─── Étape de build ────────────────────────────────────────
FROM rust:1-slim-bookworm AS builder

WORKDIR /app

# Outils de compilation pour pdf-extract (dépendances C éventuelles).
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config build-essential \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY web ./web
COPY examples ./examples

RUN cargo build --release

# ─── Image finale ──────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/workeo /usr/local/bin/workeo
COPY --from=builder /app/web ./web
COPY --from=builder /app/examples ./examples

ENV PORT=8080 \
    DATABASE_URL=sqlite://data/workeo.db \
    LLM_PROVIDER=ollama \
    OLLAMA_BASE_URL=http://host.docker.internal:11434 \
    WEB_DIR=/app/web

VOLUME ["/app/data"]
EXPOSE 8080

CMD ["workeo"]
