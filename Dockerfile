# Build stage
FROM rust:1.84-bookworm AS builder

WORKDIR /app

# Copy manifests first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create dummy src and bench stubs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && mkdir benches \
    && echo "fn main() {}" > benches/response_benchmarks.rs \
    && echo "fn main() {}" > benches/endpoint_benchmarks.rs \
    && cargo build --release \
    && rm -rf src benches

# Copy actual source and rebuild
COPY src ./src
COPY benches ./benches
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

LABEL org.opencontainers.image.title="rucho" \
      org.opencontainers.image.description="Lightweight HTTP/TCP/UDP echo server" \
      org.opencontainers.image.source="https://github.com/rumpus/rucho" \
      org.opencontainers.image.licenses="MIT"

# Install only runtime dependencies (CA certs for HTTPS)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -r rucho \
    && useradd -r -g rucho -s /sbin/nologin -c "Rucho service user" rucho \
    && mkdir -p /etc/rucho /var/run/rucho \
    && chown rucho:rucho /etc/rucho /var/run/rucho

# Copy binary from builder
COPY --from=builder --chown=rucho:rucho /app/target/release/rucho /usr/local/bin/rucho

# Copy config
COPY --chown=rucho:rucho config_samples/rucho.conf.default /etc/rucho/rucho.conf

EXPOSE 8080 9090

USER rucho

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/usr/local/bin/rucho", "status"]

CMD ["/usr/local/bin/rucho", "start"]
