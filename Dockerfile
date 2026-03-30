FROM lukemathwalker/cargo-chef:latest-rust-1.94.0-bookworm AS chef

WORKDIR /build

# --- Planner: generate a dependency recipe from Cargo.toml/Cargo.lock ---
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# --- Builder: cook deps (cached), then build the binary ---
FROM chef AS builder

# cmake and clang may be required by native TLS dependencies
RUN apt-get update && apt-get install -y cmake clang pkg-config && rm -rf /var/lib/apt/lists/*

COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --locked --features s3-sync --recipe-path recipe.json

COPY . .
RUN cargo build --release --locked --features s3-sync && cp target/release/pathscale_be /output

# --- Builder (no features): same as above but with no feature flags ---
FROM chef AS builder-no-features

RUN apt-get update && apt-get install -y cmake clang pkg-config && rm -rf /var/lib/apt/lists/*

COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --locked --recipe-path recipe.json

COPY . .
RUN cargo build --release --locked && cp target/release/pathscale_be /output

# --- Rclone binary ---
FROM rclone/rclone:latest@sha256:5008198051d87f3143e29df338b1950fc17cedbf0ee60088316d2404f5dd7567 AS rclone-bin

# --- Runtime ---
FROM debian:bookworm-slim@sha256:8af0e5095f9964007f5ebd11191dfe52dcb51bf3afa2c07f055fc5451b78ba0e AS runtime

RUN apt-get update && \
    apt-get install -y --no-install-recommends curl jq ca-certificates certbot openssl python3-pip && \
    rm -rf /var/lib/apt/lists/*

COPY --from=rclone-bin /usr/local/bin/rclone /usr/local/bin/rclone

RUN pip3 install certbot-dns-bunny --break-system-packages

COPY --from=builder /output /usr/local/bin/pathscale_be
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]

# --- Runtime (fly.io + Tigris FUSE mount) — plain HTTP, tigrisfs filesystem mount ---
# Tigris bucket is mounted at /mnt/tigris via tigrisfs (FUSE). The app writes to
# the local mount path; tigrisfs handles all S3 I/O transparently.
# fly.io sets AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY, AWS_ENDPOINT_URL_S3, and
# BUCKET_NAME automatically when a Tigris storage resource is attached.
# Requires [experimental] privileged = true in fly.toml for FUSE support.
FROM debian:bookworm-slim@sha256:8af0e5095f9964007f5ebd11191dfe52dcb51bf3afa2c07f055fc5451b78ba0e AS runtime-fly

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates fuse3 curl \
    && ARCH=$(uname -m) \
    && if [ "$ARCH" = "x86_64" ]; then ARCH="amd64"; fi \
    && if [ "$ARCH" = "aarch64" ]; then ARCH="arm64"; fi \
    && VERSION=$(curl -s https://api.github.com/repos/tigrisdata/tigrisfs/releases/latest | grep -o '"tag_name": "[^"]*' | cut -d'"' -f4) \
    && curl -L "https://github.com/tigrisdata/tigrisfs/releases/download/${VERSION}/tigrisfs_${VERSION#v}_linux_${ARCH}.tar.gz" -o /tmp/tigrisfs.tar.gz \
    && tar -xzf /tmp/tigrisfs.tar.gz -C /usr/local/bin/ \
    && rm /tmp/tigrisfs.tar.gz \
    && chmod +x /usr/local/bin/tigrisfs \
    && apt-get purge -y --auto-remove curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder-no-features /output /usr/local/bin/pathscale_be

RUN printf '#!/bin/sh\nset -e\n\nmkdir -p /mnt/tigris\necho "Mounting Tigris bucket ${BUCKET_NAME}..."\n/usr/local/bin/tigrisfs --endpoint "${AWS_ENDPOINT_URL_S3}" "${BUCKET_NAME}" /mnt/tigris &\nsleep 3\n\nexec /usr/local/bin/pathscale_be\n' \
    > /startup.sh && chmod +x /startup.sh

ENTRYPOINT ["/startup.sh"]