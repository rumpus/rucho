FROM ubuntu:latest

ENV PATH="/root/.cargo/bin:${PATH}"
RUN apt-get update && apt-get install -y curl build-essential jq && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

EXPOSE 8080
EXPOSE 9090

CMD ["/app/target/release/echo-server", "start"]
