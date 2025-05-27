FROM ubuntu:latest

ENV PATH="/root/.cargo/bin:${PATH}"
RUN apt-get update && apt-get install -y curl build-essential jq && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

# Create directory for documentation and copy sample configuration
RUN mkdir -p /usr/share/doc/rucho/examples
COPY config_samples/rucho.conf.default /usr/share/doc/rucho/examples/rucho.conf.default

EXPOSE 8080
EXPOSE 9090

CMD ["/app/target/release/echo-server", "start"]
