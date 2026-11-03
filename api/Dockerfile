FROM rust:1.95 AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY api/ .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/app/target \
    cargo chef cook --release --recipe-path recipe.json
COPY api/ .
RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/app/target \
    cargo build --release && cp /app/target/release/api /app/api-server

FROM debian:trixie-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/api-server /usr/local/bin/api
COPY catalog/ /app/catalog/
ENV CATALOG_DIR=/app/catalog BIND=0.0.0.0:8080
EXPOSE 8080
USER nobody
CMD ["api"]
