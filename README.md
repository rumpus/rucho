# Rucho - HTTP Echo Server

Simple, fast, and scalable HTTP echo server built with Rust and Axum.
Designed for testing, debugging, and simulating various HTTP behaviors.

## Features

- HTTP echo endpoints for all major methods (GET, POST, PUT, PATCH, DELETE, OPTIONS, HEAD)
- Dynamic HTTP status simulation (`/status/:code`)
- Configurable response delay (`/delay/:n`, max 300s)
- TCP and UDP echo listeners for protocol testing
- HTTPS support via Rustls with HTTP/2
- OpenAPI/Swagger documentation
- CLI for server management (start, stop, status)
- Configuration via files and environment variables
- Docker and systemd support
- Graceful shutdown handling

## Quick Start

```bash
# Clone the repository
git clone https://github.com/rumpus/rucho.git
cd rucho

# Build
cargo build --release

# Start the server
./target/release/rucho start

# Or run directly with cargo
cargo run -- start
```

Server runs at `http://localhost:8080` by default.

## CLI Commands

```bash
rucho start    # Start the server
rucho stop     # Stop the server
rucho status   # Check server status
rucho version  # Display version
```

## API Endpoints

| Method  | Path              | Description                                          |
|---------|-------------------|------------------------------------------------------|
| GET     | `/`               | Welcome message                                      |
| GET     | `/get`            | Echo request details                                 |
| HEAD    | `/get`            | Headers only                                         |
| POST    | `/post`           | Echo request with JSON body                          |
| PUT     | `/put`            | Echo request with JSON body                          |
| PATCH   | `/patch`          | Echo request with JSON body                          |
| DELETE  | `/delete`         | Echo request details                                 |
| OPTIONS | `/options`        | Return allowed methods                               |
| ANY     | `/status/:code`   | Return specified HTTP status code                    |
| ANY     | `/anything`       | Echo any request                                     |
| ANY     | `/anything/*path` | Echo any request with path                           |
| ANY     | `/delay/:n`       | Delay response by n seconds (max 300)                |
| GET     | `/healthz`        | Health check                                         |
| GET     | `/endpoints`      | List all endpoints                                   |
| GET     | `/swagger-ui`     | OpenAPI documentation                                |

### Query Parameters

All echo endpoints support `?pretty=true` for formatted JSON output.

## Configuration

### Configuration Files

Rucho loads configuration in this order (later overrides earlier):

1. Hardcoded defaults
2. `/etc/rucho/rucho.conf` (system-wide)
3. `./rucho.conf` (local directory)
4. Environment variables (`RUCHO_*`)

### Parameters

| Parameter                   | Default              | Env Variable                   | Description                    |
|-----------------------------|----------------------|--------------------------------|--------------------------------|
| `prefix`                    | `/usr/local/rucho`   | `RUCHO_PREFIX`                 | Installation prefix            |
| `log_level`                 | `info`               | `RUCHO_LOG_LEVEL`              | Log level (trace/debug/info/warn/error) |
| `server_listen_primary`     | `0.0.0.0:8080`       | `RUCHO_SERVER_LISTEN_PRIMARY`  | Primary HTTP listener          |
| `server_listen_secondary`   | `0.0.0.0:9090`       | `RUCHO_SERVER_LISTEN_SECONDARY`| Secondary HTTP listener        |
| `server_listen_tcp`         | (none)               | `RUCHO_SERVER_LISTEN_TCP`      | TCP echo listener address      |
| `server_listen_udp`         | (none)               | `RUCHO_SERVER_LISTEN_UDP`      | UDP echo listener address      |
| `ssl_cert`                  | (none)               | `RUCHO_SSL_CERT`               | Path to SSL certificate        |
| `ssl_key`                   | (none)               | `RUCHO_SSL_KEY`                | Path to SSL private key        |

### HTTPS Configuration

To enable HTTPS, add `ssl` suffix to the listen address:

```ini
server_listen_primary = 0.0.0.0:443 ssl
ssl_cert = /path/to/cert.pem
ssl_key = /path/to/key.pem
```

### TCP/UDP Echo Listeners

Enable TCP and/or UDP echo servers for protocol testing:

```ini
server_listen_tcp = 0.0.0.0:7777
server_listen_udp = 0.0.0.0:7778
```

Test with:
```bash
# TCP
echo "hello" | nc localhost 7777

# UDP
echo "hello" | nc -u localhost 7778
```

## Project Structure

```
src/
├── main.rs              # Application entrypoint
├── lib.rs               # Library exports
├── cli/                 # CLI argument parsing and commands
│   ├── mod.rs
│   └── commands.rs      # start, stop, status, version handlers
├── routes/              # HTTP route handlers
│   ├── mod.rs
│   ├── core_routes.rs   # Core echo endpoints
│   ├── delay.rs         # /delay/:n endpoint
│   └── healthz.rs       # /healthz endpoint
├── server/              # Server setup and orchestration
│   ├── mod.rs
│   ├── http.rs          # HTTP/HTTPS listener setup
│   ├── tcp.rs           # TCP echo listener
│   ├── udp.rs           # UDP echo listener
│   └── shutdown.rs      # Graceful shutdown handling
├── tcp_udp_handlers.rs  # TCP/UDP echo protocol handlers
└── utils/               # Utility modules
    ├── mod.rs
    ├── config.rs        # Configuration loading
    ├── constants.rs     # Centralized constants
    ├── error_response.rs
    ├── json_response.rs
    ├── pid.rs           # PID file management
    ├── request_models.rs
    └── server_config.rs # Listener and TLS configuration
```

## Docker

### Build and Run

```bash
docker build -t rucho .
docker run -p 8080:8080 -p 9090:9090 rucho
```

### Docker Compose

```bash
docker-compose up -d
```

Configure via environment variables in `docker-compose.yml`:

```yaml
services:
  rucho:
    environment:
      RUCHO_LOG_LEVEL: "debug"
      RUCHO_SERVER_LISTEN_TCP: "0.0.0.0:7777"
```

## Systemd

When installed via `.deb` package, the systemd service is automatically enabled:

```bash
sudo systemctl status rucho
sudo systemctl stop rucho
sudo systemctl start rucho
sudo systemctl restart rucho
```

## Examples

```bash
# Simple GET
curl http://localhost:8080/get

# POST with JSON body
curl -X POST http://localhost:8080/post \
  -H "Content-Type: application/json" \
  -d '{"key": "value"}'

# Pretty-printed response
curl "http://localhost:8080/get?pretty=true"

# Simulate 503 error
curl -i http://localhost:8080/status/503

# Delayed response (5 seconds)
curl http://localhost:8080/delay/5

# Health check
curl http://localhost:8080/healthz
```

## Tech Stack

- [Rust](https://www.rust-lang.org/)
- [Axum](https://docs.rs/axum/latest/axum/) - Web framework
- [Tokio](https://tokio.rs/) - Async runtime
- [Tower-HTTP](https://docs.rs/tower-http/latest/tower_http/) - HTTP middleware
- [Rustls](https://docs.rs/rustls/latest/rustls/) - TLS implementation
- [utoipa](https://docs.rs/utoipa/latest/utoipa/) - OpenAPI generation

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the [MIT License](LICENSE).
