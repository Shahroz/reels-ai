# Dockerfile for building the narrativ backend as part of the monorepo workspace.
# Use: docker build -f build/reels/Dockerfile -t reels-service .

FROM rustlang/rust:nightly as builder
WORKDIR /usr/src/reels

# Copy manifests first to leverage Docker caching.
COPY Cargo.toml Cargo.lock ./
COPY .cargo/config.toml ./.cargo/config.toml
COPY crates/reels/backend/Cargo.toml crates/reels/backend/Cargo.toml
COPY crates/reels/backend/src crates/reels/backend/src
COPY crates/llm/Cargo.toml crates/llm/Cargo.toml
COPY crates/llm/src crates/llm/src
COPY crates/api_clients/Cargo.toml crates/api_clients/Cargo.toml
COPY crates/api_clients/src crates/api_clients/src

# --- MODIFICATION: Fetch dependencies using the mounted secrets ---
# The --mount flags securely provide the config files for this command only.
RUN --mount=type=secret,id=gitconfig,target=/root/.gitconfig \
    cargo fetch

# Copy the rest of the source code
COPY . .

WORKDIR /usr/src/reels/crates/reels/backend
ENV SQLX_OFFLINE=true

RUN cargo build --release

FROM debian:trixie-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 imagemagick libheif-examples \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /usr/src/reels/target/release/backend /usr/local/bin/backend
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/backend"]