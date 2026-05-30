# Rucho - HTTP Echo Server

[![CI](https://github.com/rumpus/rucho/actions/workflows/ci.yml/badge.svg)](https://github.com/rumpus/rucho/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/rumpus/rucho)](https://github.com/rumpus/rucho/releases/latest)

Simple, fast, and scalable HTTP echo server built with Rust and Axum.
Designed for testing and debugging HTTP behaviors — and for use as a controllable
upstream behind Kong Gateway or inside Kong Mesh.

## Why rucho?

Rucho is an **echo server first** — a faster, more robust alternative to [httpbin](https://httpbin.org) for inspecting and shaping HTTP requests and responses:

- **Fast & robust** — Rust / Axum / Tokio with zero-copy responses; no interpreter startup and predictable latency under load (vs. Python/Flask httpbin).
- **Maintained & tested** — the original httpbin is effectively unmaintained; rucho ships an integration + unit test suite and a multi-platform (Linux + Windows) CI matrix.
- **More than HTTP** — built-in **TCP and UDP** echo listeners, first-class **HTTPS / HTTP-2** via Rustls, and connection/socket tuning that go-httpbin doesn't expose.
- **Chaos built in** — failure / delay / corruption injection for resilience testing, no sidecar required.

It's also purpose-built as a **controllable testing upstream behind [Kong Gateway](https://konghq.com/products/kong-gateway) or inside [Kong Mesh](https://konghq.com/products/kong-mesh)**: endpoints that deterministically emit the stimuli (status codes, delays, slow streams, redirects, forced content types, byte ranges) you need to observe how the gateway/mesh proxies, transforms, times out, retries, and routes. See **[Using rucho as a Kong upstream](docs/USAGE_EXAMPLES.md#using-rucho-as-a-kong-upstream)**.

## Features

- HTTP echo endpoints for all major methods (GET, POST, PUT, PATCH, DELETE, OPTIONS, HEAD)
- Dynamic HTTP status simulation (`/status/:code`)
- Configurable response delay (`/delay/:n`, max 300s)
- Chained HTTP redirects (`/redirect/:n`, max 20 hops)
- Cookie inspection, setting, and deletion (`/cookies`, `/cookies/set`, `/cookies/delete`)
- Base64 decoding with UTF-8 detection (`/base64/:encoded`, max 4 KiB)
- Non-JSON content types (`/xml` → `application/xml`, `/html` → `text/html`) for testing gateway content-type handling
- Sample images (`/image/:format` — png, jpeg, svg, webp) for testing gateway binary/image handling
- Byte-range requests (`/range/:n` — `Accept-Ranges`, 206 partial content) for testing gateway range/resumable-download handling
- Forced content encodings (`/gzip`, `/deflate`, `/brotli`) — return a body in that `Content-Encoding` regardless of `Accept-Encoding`, for testing gateway decode/transform behavior
- Conditional caching (`/cache` → `ETag`/`Last-Modified` + `304`; `/cache/:n` → `Cache-Control: max-age`) for testing gateway cache behavior
- Gateway plugin-testing trio:
  - `/response-headers?key=value` — echo query params as response headers
  - `/bytes/:n` — random bytes as `application/octet-stream` (max 10 MiB)
  - `/drip?duration=N&numbytes=M` — slow byte stream for inter-byte timeout testing
- Configurable request body size cap (`max_body_size_bytes`, default 2 MiB)
- TCP and UDP echo listeners for protocol testing
- HTTPS support via Rustls with HTTP/2
- Response compression (gzip, brotli) - optional, client-negotiated
- Connection keep-alive tuning (TCP keep-alive, TCP_NODELAY, header timeout)
- Chaos engineering mode for resilience testing
- Request timing in JSON responses (`timing.duration_ms`)
- `X-Request-Id` correlation header on every response — propagates a non-blank inbound id (e.g. from a mesh sidecar), else mints a UUID v4 (`request_id_enabled`, default on)
- OpenAPI/Swagger documentation
- CLI for server management (start, stop, status)
- Configuration via files and environment variables
- Docker and systemd support
- Graceful shutdown on SIGINT + SIGTERM (drains in-flight requests; container/mesh-friendly)

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
| ANY     | `/status/:code`   | Return a status code + `{status, reason}` JSON body  |
| ANY     | `/anything`       | Echo any request                                     |
| ANY     | `/anything/*path` | Echo any request with path                           |
| ANY     | `/delay/:n`       | Delay response by n seconds (max 300)                |
| ANY     | `/redirect/:n`    | Chain of n 302s (max 20; `X-Redirect-Count` header)  |
| GET     | `/cookies`        | Inspect request cookies                              |
| GET     | `/cookies/set`    | Set cookies (+ secure/httponly/samesite/max_age)     |
| GET     | `/cookies/delete` | Delete cookies via query params and redirect         |
| GET     | `/base64/:encoded`| Decode URL-safe base64 (max 4096 bytes)              |
| GET     | `/bytes/:n`       | Return n random bytes (max 10 MiB)                   |
| GET     | `/response-headers`| Echo query params as response headers + JSON body   |
| GET     | `/drip`           | Slowly stream bytes (test inter-byte timeouts)       |
| GET     | `/xml`            | Sample XML document (`application/xml`)              |
| GET     | `/html`           | Sample HTML document (`text/html`)                   |
| GET     | `/image/:format`  | Sample image (png, jpeg, svg, webp)                  |
| GET     | `/range/:n`       | n bytes w/ Range support (206 partial content)       |
| GET     | `/gzip`           | gzip-encoded JSON echo (forced `Content-Encoding`)   |
| GET     | `/deflate`        | deflate-encoded JSON echo (forced encoding)          |
| GET     | `/brotli`         | brotli-encoded JSON echo (forced encoding)           |
| GET     | `/cache`          | 304 on conditional req; else ETag + Last-Modified    |
| GET     | `/cache/:n`       | `Cache-Control: public, max-age=n`                   |
| GET     | `/uuid`           | Random UUID v4                                       |
| GET     | `/ip`             | Client IP address                                    |
| GET     | `/user-agent`     | User-Agent header echo                               |
| GET     | `/headers`        | All request headers                                  |
| GET     | `/healthz`        | Health check                                         |
| GET     | `/metrics`        | Request statistics (when enabled)                    |
| GET     | `/endpoints`      | List all endpoints                                   |
| GET     | `/swagger-ui`     | OpenAPI documentation                                |

### JSON Output

All JSON responses are pretty-printed by default for readability.

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
| `log_format`                | `text`               | `RUCHO_LOG_FORMAT`            | Log output: `text` or `json` (structured) |
| `pid_file`                  | `/var/run/rucho/rucho.pid` | `RUCHO_PID_FILE`         | PID file path (write is non-fatal) |
| `server_listen_primary`     | `0.0.0.0:8080`       | `RUCHO_SERVER_LISTEN_PRIMARY`  | Primary HTTP listener          |
| `server_listen_secondary`   | `0.0.0.0:9090`       | `RUCHO_SERVER_LISTEN_SECONDARY`| Secondary HTTP listener        |
| `server_listen_tcp`         | (none)               | `RUCHO_SERVER_LISTEN_TCP`      | TCP echo listener address      |
| `server_listen_udp`         | (none)               | `RUCHO_SERVER_LISTEN_UDP`      | UDP echo listener address      |
| `ssl_cert`                  | (none)               | `RUCHO_SSL_CERT`               | Path to SSL certificate        |
| `ssl_key`                   | (none)               | `RUCHO_SSL_KEY`                | Path to SSL private key        |
| `metrics_enabled`           | `false`              | `RUCHO_METRICS_ENABLED`        | Enable /metrics endpoint       |
| `compression_enabled`       | `false`              | `RUCHO_COMPRESSION_ENABLED`    | Enable gzip/brotli compression |
| `request_id_enabled`        | `true`               | `RUCHO_REQUEST_ID_ENABLED`     | X-Request-Id header on responses |
| `http_keep_alive_timeout`   | `75`                 | `RUCHO_HTTP_KEEP_ALIVE_TIMEOUT`| HTTP idle connection timeout (seconds) |
| `tcp_keepalive_time`        | `60`                 | `RUCHO_TCP_KEEPALIVE_TIME`     | TCP keepalive idle time (seconds) |
| `tcp_keepalive_interval`    | `15`                 | `RUCHO_TCP_KEEPALIVE_INTERVAL` | TCP keepalive probe interval (seconds) |
| `tcp_keepalive_retries`     | `5`                  | `RUCHO_TCP_KEEPALIVE_RETRIES`  | TCP keepalive probe retries (1-10) |
| `tcp_nodelay`               | `true`               | `RUCHO_TCP_NODELAY`            | Disable Nagle's algorithm |
| `header_read_timeout`       | `30`                 | `RUCHO_HEADER_READ_TIMEOUT`    | Max time to read request headers (seconds) |
| `chaos_mode`                | (none)               | `RUCHO_CHAOS_MODE`             | Enable [chaos types](#chaos-engineering-mode) |

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
benches/                         # Criterion benchmarks (cargo bench)
├── response_benchmarks.rs       # Response building microbenchmarks
└── endpoint_benchmarks.rs       # Full request cycle benchmarks
debian/man/                      # Debian package extras
└── rucho.1                      # Man page (man rucho)
tests/                           # Integration tests (cargo test)
└── integration.rs               # HTTP integration tests (reqwest)
src/
├── main.rs              # Application entrypoint
├── lib.rs               # Library exports
├── cli/                 # CLI argument parsing and commands
│   ├── mod.rs
│   └── commands.rs      # start, stop, status, version handlers
├── routes/              # HTTP route handlers
│   ├── mod.rs
│   ├── base64.rs        # /base64/:encoded endpoint
│   ├── bytes.rs         # /bytes/:n endpoint
│   ├── cache.rs         # /cache + /cache/:n endpoints
│   ├── content_types.rs # /xml + /html endpoints
│   ├── cookies.rs       # /cookies endpoints
│   ├── core_routes.rs   # Core echo + utility endpoints
│   ├── delay.rs         # /delay/:n endpoint
│   ├── drip.rs          # /drip slow-streaming endpoint
│   ├── encoding.rs      # /gzip + /deflate + /brotli endpoints
│   ├── healthz.rs       # /healthz endpoint
│   ├── image.rs         # /image/:format endpoint
│   ├── response_headers.rs # /response-headers endpoint
│   ├── metrics.rs       # /metrics endpoint handler
│   ├── range.rs         # /range/:n endpoint
│   └── redirect.rs      # /redirect/:n endpoint
├── server/              # Server setup and orchestration
│   ├── mod.rs
│   ├── chaos_layer.rs   # Chaos engineering middleware
│   ├── http.rs          # HTTP/HTTPS listener setup
│   ├── metrics_layer.rs # Metrics collection middleware
│   ├── tcp.rs           # TCP echo listener
│   ├── timing_layer.rs  # Request timing middleware
│   ├── udp.rs           # UDP echo listener
│   └── shutdown.rs      # Graceful shutdown handling
├── tcp_udp_handlers.rs  # TCP/UDP echo protocol handlers
└── utils/               # Utility modules
    ├── mod.rs
    ├── config.rs        # Configuration loading
    ├── constants.rs     # Centralized constants
    ├── error_response.rs
    ├── json_response.rs
    ├── metrics.rs       # Metrics data structures
    ├── pid.rs           # PID file management
    ├── server_config.rs # Listener and TLS configuration
    └── timing.rs        # Timing utilities
```

## Docker

### Pull from Docker Hub

```bash
docker pull rumpus/rucho:latest
docker run -p 8080:8080 -p 9090:9090 rumpus/rucho:latest
```

### Build Locally

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

### Response Compression

Enable optional response compression:

```ini
compression_enabled = true
```

Or via environment variable: `RUCHO_COMPRESSION_ENABLED=true`

When enabled, responses are compressed based on the client's `Accept-Encoding` header:
- `Accept-Encoding: gzip` → gzip compression
- `Accept-Encoding: br` → brotli compression

### Connection Keep-Alive Tuning

Rucho configures TCP and HTTP connection settings for lower latency, faster dead-connection detection, and protection against slow clients. All settings have sensible defaults and can be overridden via config file or environment variables.

- **TCP keep-alive** sends probe packets on idle connections to detect crashed peers (~90s detection vs OS default ~2 hours)
- **TCP_NODELAY** disables Nagle's algorithm, eliminating buffering delays for small echo responses (enabled by default)
- **Header read timeout** protects against slowloris-style attacks by closing connections that send headers too slowly

```ini
# Example: aggressive tuning for high-traffic environments
http_keep_alive_timeout = 30
tcp_keepalive_time = 30
tcp_keepalive_interval = 10
tcp_keepalive_retries = 3
tcp_nodelay = true
header_read_timeout = 15
```

### Chaos Engineering Mode

Enable chaos mode to randomly inject failures, delays, and response corruption for resilience testing. Each chaos type rolls independently against its configured probability per request. Disabled by default.

#### Chaos Parameters

| Parameter               | Default | Env Variable                  | Description                                          |
|-------------------------|---------|-------------------------------|------------------------------------------------------|
| `chaos_mode`            | (none)  | `RUCHO_CHAOS_MODE`            | Chaos types to enable (comma-separated: `failure`, `delay`, `corruption`) |
| `chaos_failure_rate`    | `0.0`   | `RUCHO_CHAOS_FAILURE_RATE`    | Probability of failure injection (0.01-1.0)          |
| `chaos_failure_codes`   | (none)  | `RUCHO_CHAOS_FAILURE_CODES`   | HTTP status codes to return (comma-separated, 400-599) |
| `chaos_delay_rate`      | `0.0`   | `RUCHO_CHAOS_DELAY_RATE`      | Probability of delay injection (0.01-1.0)            |
| `chaos_delay_ms`        | (none)  | `RUCHO_CHAOS_DELAY_MS`        | Delay in ms, or `random` for random delays           |
| `chaos_delay_max_ms`    | `0`     | `RUCHO_CHAOS_DELAY_MAX_MS`    | Max delay in ms (required when `chaos_delay_ms=random`) |
| `chaos_corruption_rate` | `0.0`   | `RUCHO_CHAOS_CORRUPTION_RATE` | Probability of response corruption (0.01-1.0)        |
| `chaos_corruption_type` | (none)  | `RUCHO_CHAOS_CORRUPTION_TYPE` | Corruption type: `empty`, `truncate`, or `garbage`   |
| `chaos_inform_header`   | `true`  | `RUCHO_CHAOS_INFORM_HEADER`   | Add `X-Chaos` header to affected responses           |

#### Usage Examples

**Failure injection** — randomly return 500/503 errors on 10% of requests:

```ini
chaos_mode = failure
chaos_failure_rate = 0.1
chaos_failure_codes = 500,503
```

**Delay injection** — add random delays (up to 5s) on 20% of requests:

```ini
chaos_mode = delay
chaos_delay_rate = 0.2
chaos_delay_ms = random
chaos_delay_max_ms = 5000
```

**Response corruption** — truncate response bodies on 5% of requests:

```ini
chaos_mode = corruption
chaos_corruption_rate = 0.05
chaos_corruption_type = truncate
```

**Combined** — enable multiple chaos types simultaneously:

```ini
chaos_mode = failure,delay,corruption
chaos_failure_rate = 0.1
chaos_failure_codes = 500,502,503
chaos_delay_rate = 0.2
chaos_delay_ms = random
chaos_delay_max_ms = 5000
chaos_corruption_rate = 0.05
chaos_corruption_type = empty
```

Affected responses include an `X-Chaos` header listing which chaos types were applied (e.g., `X-Chaos: delay,corruption`). Disable this with `chaos_inform_header = false`.

## Examples

For comprehensive examples with curl, Python, and JavaScript, see [docs/USAGE_EXAMPLES.md](docs/USAGE_EXAMPLES.md). For detailed endpoint documentation, see the [API Reference](docs/API_REFERENCE.md).

```bash
# Simple GET
curl http://localhost:8080/get

# POST with JSON body
curl -X POST http://localhost:8080/post \
  -H "Content-Type: application/json" \
  -d '{"key": "value"}'

# Simulate 503 error
curl -i http://localhost:8080/status/503

# Delayed response (5 seconds)
curl http://localhost:8080/delay/5

# Inspect cookies
curl -b "session=abc123; theme=dark" http://localhost:8080/cookies

# Set cookies
curl -c - http://localhost:8080/cookies/set?name=rucho&lang=rust

# Delete cookies
curl -b "name=rucho" -c - http://localhost:8080/cookies/delete?name

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
