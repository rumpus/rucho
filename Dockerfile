# Build stage
FROM rust:1.84-bookworm AS builder

WORKDIR /app

# Copy manifests first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create dummy src to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy actual source and rebuild
COPY src ./src
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install only runtime dependencies (CA certs for HTTPS)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -r rucho \
    && useradd -r -g rucho -s /sbin/nologin -c "Rucho service user" rucho \
    && mkdir -p /etc/rucho /var/run/rucho \
    && chown rucho:rucho /etc/rucho /var/run/rucho

# Copy binary from builder
COPY --from=builder /app/target/release/rucho /usr/local/bin/rucho

# Copy config
COPY config_samples/rucho.conf.default /etc/rucho/rucho.conf

RUN chown rucho:rucho /usr/local/bin/rucho /etc/rucho/rucho.conf

EXPOSE 8080 9090

USER rucho

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/usr/local/bin/rucho", "status"]

CMD ["/usr/local/bin/rucho", "start"]
