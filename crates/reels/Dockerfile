# Stage 1: Build the Rust backend
FROM rust:1.82-alpine3.20 as app-build
WORKDIR /build

# Install dependencies and tools
RUN apk add --no-cache musl-dev elfutils xz wget pkgconfig libressl-dev perl make && \
    wget https://github.com/upx/upx/releases/download/v4.0.2/upx-4.0.2-amd64_linux.tar.xz && \
    unxz upx-4.0.2-amd64_linux.tar.xz && tar xvf upx-4.0.2-amd64_linux.tar && \
    cp upx-4.0.2-amd64_linux/upx /usr/bin/upx && chmod +x /usr/bin/upx

# Copy pre-fetched dependencies from the build context
COPY ./.cargo ./.cargo
COPY ./target ./target

COPY backend/ /build/backend/

# Set working directory for backend build
WORKDIR /build/backend

# Build the Rust project
RUN cargo build --release && \
    eu-elfcompress target/release/backend && \
    strip target/release/backend && \
    upx -9 --lzma target/release/backend && \
    chmod +x target/release/backend

# Stage 3: Prepare the final image
FROM alpine:3.20
WORKDIR /app

RUN apk add --no-cache libressl imagemagick libheif && \
    adduser -h /app -D appuser && \
    chmod 700 /app && \
    chown -R appuser: /app

COPY --from=app-build /build/backend/target/release/backend /app/backend
RUN chown -R appuser: /app && chmod +x /app/backend

USER appuser
EXPOSE 8081
CMD ["/app/backend"]