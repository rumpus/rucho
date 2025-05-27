FROM ubuntu:latest

ENV PATH="/root/.cargo/bin:${PATH}"

# Install dependencies, Rust, create user/group
RUN apt-get update && apt-get install -y curl build-essential jq &&     curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y &&     groupadd -r rucho &&     useradd -r -g rucho -s /sbin/nologin -c "Rucho service user" rucho &&     apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy application source and build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# Install binary, create config/doc/run directories, set initial ownership
RUN cp /app/target/release/rucho /usr/local/bin/rucho &&     mkdir -p /etc/rucho &&     mkdir -p /usr/share/doc/rucho/examples &&     mkdir -p /var/run/rucho &&     chown rucho:rucho /usr/local/bin/rucho &&     chown rucho:rucho /etc/rucho &&     chown rucho:rucho /var/run/rucho

# Copy configuration files
COPY config_samples/rucho.conf.default /usr/share/doc/rucho/examples/rucho.conf.default
COPY config_samples/rucho.conf.default /etc/rucho/rucho.conf

# Set ownership for copied active configuration file
RUN chown rucho:rucho /etc/rucho/rucho.conf

# Clean up build artifacts to reduce image size
RUN rm -rf /app/target /root/.cargo

EXPOSE 8080
EXPOSE 9090

USER rucho
CMD ["/usr/local/bin/rucho", "start"]
