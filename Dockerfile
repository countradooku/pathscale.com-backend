FROM lukemathwalker/cargo-chef:latest-rust-1.94.0-bookworm AS chef

WORKDIR /build

# --- Planner: generate a dependency recipe from Cargo.toml/Cargo.lock ---
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# --- Builder: cook deps (cached), then build the binary ---
FROM chef AS builder

RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev cmake clang && rm -rf /var/lib/apt/lists/*

COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --locked --recipe-path recipe.json

COPY . .
RUN cargo build --release --locked --bin pathscale_be

# --- Runtime ---
FROM debian:bookworm-slim@sha256:8af0e5095f9964007f5ebd11191dfe52dcb51bf3afa2c07f055fc5451b78ba0e AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates e2fsprogs && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/pathscale_be /usr/local/bin/pathscale_be
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

EXPOSE 8080
ENTRYPOINT ["/entrypoint.sh"]
