# Stage 1: Build the React Frontend
FROM node:lts-alpine as frontend-builder

WORKDIR /app/frontend

# Copy package files and install dependencies
COPY frontend/package.json frontend/package-lock.json* ./
# Use `ci` for cleaner installs in CI/CD environments
RUN npm ci

# Copy the rest of the frontend source code
COPY frontend/ ./

# Build the frontend application (assuming output is in 'dist')
RUN npm run build

# Stage 2: Build the Rust Backend (statically linked)
FROM rust:latest as rust-builder

# Install musl-tools for static linking
RUN apt-get update && apt-get install -y musl-tools musl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Create a dummy project to cache dependencies first
RUN USER=root cargo new --bin agentloop
WORKDIR /app/agentloop-cli

# Copy manifests and lock file
COPY Cargo.toml Cargo.lock ./
COPY crates/agentloop/Cargo.toml ./crates/agentloop/
# Copy potentially other workspace crates manifests if needed
# COPY crates/other_crate/Cargo.toml ./crates/other_crate/

# Build dummy project to cache dependencies
# Ensure target is set for caching layers correctly
RUN cargo build --release --locked --target x86_64-unknown-linux-musl

# Remove dummy source code
RUN rm -rf src

# Copy the actual source code for the workspace
COPY . .

# Copy the built frontend assets from the previous stage
# This MUST happen before the final build for rust-embed to work
COPY --from=frontend-builder /app/frontend/dist ./crates/agentloop/frontend/dist

# Build the actual application binary statically
# Ensure the correct binary name is specified if it's not the default package name
RUN cargo build --release --locked --target x86_64-unknown-linux-musl --bin agentloop

# Stage 3: Create the Final Runtime Image
FROM alpine:latest

WORKDIR /app

## Copy the statically linked binary from the builder stage
COPY --from=rust-builder /app/agentloop-cli/target/x86_64-unknown-linux-musl/release/agentloop /app/agentloop

# Copy any other necessary runtime files (e.g., config, assets) if needed
# COPY --from=rust-builder /app/config.toml /app/config.toml

# Set environment variables (if needed, defaults can be set here)
# ENV RUST_LOG=info
# ENV SERVER_ADDRESS=0.0.0.0:8080
# ENV OPENAI_API_KEY=your_key_here # Note: Use build args or runtime env vars for secrets

# Expose the port the application listens on
EXPOSE 8080

# Set the entrypoint to run the application
ENTRYPOINT ["/app/agentloop"]

# Optional: Add healthcheck if your app supports it
# HEALTHCHECK --interval=30s --timeout=3s \
#   CMD curl --fail http://localhost:8080/health || exit 1
