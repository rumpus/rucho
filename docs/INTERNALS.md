# Rucho Internals

> Comprehensive internal code documentation for the Rucho echo server.
> This document covers every module, function, data structure, and control-flow
> path in the codebase. It is intended as a developer reference — not user-facing
> API docs.
>
> **Version:** 1.4.4
> **Last updated:** 2026-02-16

---

## Table of Contents

1.  [Architecture Overview](#1-architecture-overview)
2.  [Application Startup Sequence](#2-application-startup-sequence)
3.  [Axum Router and Middleware Stack](#3-axum-router-and-middleware-stack)
    - 3.1 [Route Registration](#31-route-registration)
    - 3.2 [Middleware Layer Order](#32-middleware-layer-order)
4.  [HTTP Request Lifecycle (End-to-End Trace)](#4-http-request-lifecycle-end-to-end-trace)
5.  [Route Handlers Reference](#5-route-handlers-reference)
    - 5.1 [Endpoint Summary Table](#51-endpoint-summary-table)
    - 5.2 [Echo Handlers](#52-echo-handlers)
    - 5.3 [Utility Handlers](#53-utility-handlers)
    - 5.4 [Special Handlers](#54-special-handlers)
    - 5.5 [Infrastructure Handlers](#55-infrastructure-handlers)
6.  [Middleware Deep Dives](#6-middleware-deep-dives)
    - 6.1 [Timing Middleware](#61-timing-middleware)
    - 6.2 [Metrics Middleware](#62-metrics-middleware)
    - 6.3 [Chaos Middleware](#63-chaos-middleware)
7.  [Configuration System](#7-configuration-system)
    - 7.1 [Config and ChaosConfig Structs](#71-config-and-chaosconfig-structs)
    - 7.2 [Complete Field Reference](#72-complete-field-reference)
    - 7.3 [Loading Precedence](#73-loading-precedence)
    - 7.4 [The `load_env_var!` Macro](#74-the-load_env_var-macro)
    - 7.5 [File Parsing](#75-file-parsing)
    - 7.6 [Validation Pipeline](#76-validation-pipeline)
8.  [Server Orchestration](#8-server-orchestration)
    - 8.1 [`run_server()`](#81-run_server)
    - 8.2 [HTTP/HTTPS Setup Chain](#82-httphttps-setup-chain)
    - 8.3 [TCP Socket Configuration](#83-tcp-socket-configuration)
    - 8.4 [HTTP Builder Configuration](#84-http-builder-configuration)
    - 8.5 [TLS Configuration](#85-tls-configuration)
9.  [TCP and UDP Echo Handlers](#9-tcp-and-udp-echo-handlers)
    - 9.1 [TCP Echo Loop](#91-tcp-echo-loop)
    - 9.2 [TCP Listener Setup](#92-tcp-listener-setup)
    - 9.3 [UDP Echo with Exponential Backoff](#93-udp-echo-with-exponential-backoff)
    - 9.4 [UDP Listener Setup](#94-udp-listener-setup)
10. [Metrics Collection and Rolling Window](#10-metrics-collection-and-rolling-window)
    - 10.1 [Data Model](#101-data-model)
    - 10.2 [TimeBucket Struct](#102-timebucket-struct)
    - 10.3 [Recording Flow](#103-recording-flow)
    - 10.4 [Querying Flow](#104-querying-flow)
    - 10.5 [Snapshot Structs](#105-snapshot-structs)
11. [Process Management (PID Lifecycle)](#11-process-management-pid-lifecycle)
12. [Response Formatting](#12-response-formatting)
13. [Graceful Shutdown](#13-graceful-shutdown)
14. [OpenAPI / Swagger Integration](#14-openapi--swagger-integration)
15. [Constants Reference](#15-constants-reference)
16. [Dependency Map](#16-dependency-map)
17. [Source File Index](#17-source-file-index)
18. [Benchmark Suite](#18-benchmark-suite)

---

## 1. Architecture Overview

Rucho is a lightweight HTTP/TCP/UDP echo server built on Axum and Tokio. Its
primary purpose is echoing back request details (headers, body, method, path)
for testing and debugging API clients.

### Module Hierarchy

The crate root (`src/lib.rs:24-44`) exports five top-level modules:

```
rucho (crate root)
  |
  +-- cli/                   # Command-line interface
  |   +-- mod.rs             # Re-exports Args, CliCommand
  |   +-- commands.rs        # Args struct, CliCommand enum, command handlers
  |
  +-- routes/                # HTTP route handlers
  |   +-- mod.rs             # Re-exports submodules
  |   +-- cookies.rs         # /cookies, /cookies/set, /cookies/delete handlers + router()
  |   +-- core_routes.rs     # 16 route handlers + router()
  |   +-- delay.rs           # /delay/:n handler + router()
  |   +-- healthz.rs         # /healthz handler + router()
  |   +-- metrics.rs         # /metrics handler (stateful)
  |   +-- redirect.rs       # /redirect/:n handler + router()
  |
  +-- server/                # Server setup and orchestration
  |   +-- mod.rs             # run_server() — top-level orchestrator
  |   +-- http.rs            # HTTP/HTTPS listener setup, TCP/HTTP config
  |   +-- tcp.rs             # TCP echo listener setup
  |   +-- udp.rs             # UDP echo listener setup
  |   +-- shutdown.rs        # Ctrl+C graceful shutdown
  |   +-- chaos_layer.rs     # Chaos engineering middleware
  |   +-- metrics_layer.rs   # Metrics recording middleware
  |   +-- timing_layer.rs    # Request timing middleware
  |
  +-- tcp_udp_handlers.rs    # Raw TCP/UDP echo handlers
  |
  +-- utils/                 # Shared utilities
      +-- mod.rs             # Re-exports submodules
      +-- config.rs          # Config, ChaosConfig, loading, validation
      +-- constants.rs       # All hardcoded constants
      +-- error_response.rs  # JSON error response builder
      +-- json_response.rs   # JSON success response builder
      +-- metrics.rs         # Metrics struct, rolling window
      +-- pid.rs             # PID file management
      +-- server_config.rs   # TLS loading, address parsing
      +-- timing.rs          # RequestTiming struct
```

### Module Dependency Diagram

```
src/main.rs
  |
  +-- rucho::cli::commands  (Args, CliCommand, handle_*_command)
  +-- rucho::routes::core_routes  (router, EndpointInfo)
  +-- rucho::routes::cookies  (router, cookies_handler, set_cookies_handler, delete_cookies_handler)
  +-- rucho::routes::redirect  (router, redirect_handler)
  +-- rucho::server::chaos_layer  (chaos_middleware)
  +-- rucho::server::metrics_layer  (metrics_middleware)
  +-- rucho::server::timing_layer  (timing_middleware)
  +-- rucho::utils::config  (Config, ChaosConfig)
  +-- rucho::utils::metrics  (Metrics)
  +-- rucho::server  (run_server)
  |
  v
rucho::server::mod (run_server)
  |
  +-- server::http  (setup_http_listeners)
  |     +-- utils::config  (Config)
  |     +-- utils::server_config  (parse_listen_address, try_load_rustls_config)
  |
  +-- server::tcp  (setup_tcp_listener)
  |     +-- tcp_udp_handlers  (handle_tcp_connection)
  |
  +-- server::udp  (bind_udp_socket, setup_udp_listener)
  |     +-- tcp_udp_handlers  (handle_udp_socket)
  |
  +-- server::shutdown  (shutdown_signal)

rucho::routes::core_routes
  +-- utils::json_response  (format_json_response, format_json_response_with_timing)
  +-- utils::error_response  (format_error_response)
  +-- utils::timing  (RequestTiming)

rucho::cli::commands
  +-- utils::pid  (write_pid_file, read_pid_file, remove_pid_file, ...)
```

---

## 2. Application Startup Sequence

Entry point: `src/main.rs:74`

When you run `cargo run -- start`, the following sequence executes:

```
main()                              src/main.rs:74
  |
  +-- Args::parse()                 clap derives from CliCommand enum
  +-- Config::load()                src/utils/config.rs:684
  |     +-- Config::load_from_paths(None, None)
  |           +-- Config::load_from_paths_with_env(..., &env::var)
  |                 +-- Config::default()           hardcoded defaults
  |                 +-- read /etc/rucho/rucho.conf  (if exists)
  |                 +-- read ./rucho.conf           (if exists)
  |                 +-- apply RUCHO_* env vars via env_reader
  |
  +-- config.validate()             src/utils/config.rs:528
  |     +-- validate SSL pairs
  |     +-- validate_connection()   keep-alive bounds
  |     +-- validate_chaos()        chaos sub-config requirements
  |
  +-- tracing_subscriber init       with config.log_level
  |
  +-- match args.command
        |
        CliCommand::Start =>
          +-- handle_start_command()  src/cli/commands.rs:38
          |     +-- write_pid_file(pid)
          |
          +-- Metrics::new() (if metrics_enabled)
          +-- build_app(metrics, compression_enabled, chaos)  src/main.rs:135
          +-- run_server(&config, app)  src/server/mod.rs:24
```

### `main()` — Verbatim Source

```rust
// src/main.rs:73-128
#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = Config::load();

    // Validate configuration
    if let Err(e) = config.validate() {
        eprintln!("Configuration error: {}", e);
        std::process::exit(1);
    }

    // Initialize tracing with configured log level
    let log_level = Level::from_str(&config.log_level.to_uppercase())
        .unwrap_or_else(|_| {
            eprintln!(
                "Warning: Invalid log level '{}' in config, defaulting to INFO.",
                config.log_level
            );
            Level::INFO
        });
    tracing_subscriber::fmt().with_max_level(log_level).init();

    // Dispatch command
    match args.command {
        CliCommand::Start {} => {
            if handle_start_command() {
                let metrics = if config.metrics_enabled {
                    tracing::info!("Metrics endpoint enabled at /metrics");
                    Some(Arc::new(Metrics::new()))
                } else {
                    None
                };

                // ... logging omitted for brevity ...

                let chaos = Arc::new(config.chaos.clone());
                let app = build_app(metrics, config.compression_enabled, chaos);
                rucho::server::run_server(&config, app).await;
            }
        }
        CliCommand::Stop {} => handle_stop_command(),
        CliCommand::Status {} => handle_status_command(),
        CliCommand::Version {} => handle_version_command(),
    }
}
```

**Key points:**

- `Config::load()` happens *before* tracing is initialized — errors from config
  loading go to `eprintln!` (stderr), not tracing.
- `config.validate()` runs before anything else; exits with code 1 on failure.
- The `build_app()` call happens *inside* the `Start` branch, after the PID file
  is written.

---

## 3. Axum Router and Middleware Stack

### 3.1 Route Registration

`build_app()` at `src/main.rs:135-191` constructs the Axum `Router`:

```rust
// src/main.rs:140-146
let mut app = Router::new()
    .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
    .merge(rucho::routes::core_routes::router())  // 16 core routes
    .merge(rucho::routes::healthz::router())       // /healthz
    .merge(rucho::routes::delay::router())         // /delay/:n
    .merge(rucho::routes::redirect::router())      // /redirect/:n
    .merge(rucho::routes::cookies::router());      // /cookies, /cookies/set, /cookies/delete
```

The core routes router (`src/routes/core_routes.rs:210-241`) registers:

```rust
Router::new()
    .route("/", get(root_handler))
    .route("/get", get(get_handler))
    .route("/get", head(head_handler))
    .route("/post", post(post_handler))
    .route("/put", put(put_handler))
    .route("/patch", patch(patch_handler))
    .route("/delete", delete(delete_handler))
    .route("/options", options(options_handler))
    .route("/status/:code", any(status_handler))
    .route("/anything", any(anything_handler))
    .route("/anything/*path", any(anything_handler))
    .route("/uuid", get(uuid_handler))
    .route("/ip", get(ip_handler))
    .route("/user-agent", get(user_agent_handler))
    .route("/headers", get(headers_handler))
    .route("/endpoints", get(endpoints_handler))
```

Conditional routes added in `build_app()`:

- **`/metrics`** (GET) — only if `config.metrics_enabled` is true (`src/main.rs:149-159`)
- **Metrics middleware** — wraps all routes when metrics is enabled
- **Chaos middleware** — wraps routes when `chaos.is_enabled()` (`src/main.rs:164-171`)

### 3.2 Middleware Layer Order

Axum applies `.layer()` calls in reverse order — the **last** `.layer()` added
is the **outermost** (first to see the request). The actual execution order for
an inbound request is:

```
                         INBOUND REQUEST
                              |
                              v
  +------------------------------------------------------+
  |  NormalizePathLayer  (trim trailing slashes)          |  outermost
  +------------------------------------------------------+
                              |
                              v
  +------------------------------------------------------+
  |  CorsLayer::permissive()  (add CORS headers)         |
  +------------------------------------------------------+
                              |
                              v
  +------------------------------------------------------+
  |  CompressionLayer  (gzip/brotli, if enabled)         |
  +------------------------------------------------------+
                              |
                              v
  +------------------------------------------------------+
  |  TraceLayer  (request/response logging)               |
  +------------------------------------------------------+
                              |
                              v
  +------------------------------------------------------+
  |  timing_middleware  (inject RequestTiming extension)   |
  +------------------------------------------------------+
                              |
                              v
  +------------------------------------------------------+
  |  chaos_middleware  (failure/delay/corruption, if on)   |
  +------------------------------------------------------+
                              |
                              v
  +------------------------------------------------------+
  |  metrics_middleware  (record path + status, if on)     |
  +------------------------------------------------------+
                              |
                              v
  +------------------------------------------------------+
  |  Route Handler  (e.g., get_handler, post_handler)     |  innermost
  +------------------------------------------------------+
                              |
                              v
                        OUTBOUND RESPONSE
         (walks back up through each layer in reverse)
```

**Why this order matters:**

- Timing wraps chaos, so `duration_ms` *honestly reflects* chaos-injected
  delays.
- Metrics sits innermost (closest to the handler) so it records the actual
  status code returned by the handler (or chaos failure).
- Compression wraps everything so the final response body gets compressed.
- NormalizePath is outermost so `/get/` becomes `/get` before any routing.

The relevant code from `build_app()` (`src/main.rs:160-191`):

```rust
// Middleware order (innermost to outermost):
// routes -> chaos -> timing -> trace -> compression -> cors -> normalize-path
let app = if chaos.is_enabled() {
    app.layer(middleware::from_fn(move |req, next| {
        let chaos = chaos.clone();
        async move { chaos_middleware(req, next, chaos).await }
    }))
} else {
    app
};

let app = app.layer(middleware::from_fn(timing_middleware)).layer(
    TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO)),
);

let app = if compression_enabled {
    app.layer(CompressionLayer::new())
} else {
    app
};

app.layer(CorsLayer::permissive())
    .layer(NormalizePathLayer::trim_trailing_slash())
```

---

## 4. HTTP Request Lifecycle (End-to-End Trace)

This section traces a `GET /get` request from TCP accept through to the JSON
response sent back to the client. Assume all features are enabled (metrics,
compression, chaos).

### Step 1: TCP Accept and HTTP Parsing

The `axum_server` crate (built on `hyper`) accepts the TCP connection. The
socket has been configured with keep-alive and nodelay via
`configure_tcp_socket()` (`src/server/http.rs:18-36`). Hyper's HTTP/1.1 parser
reads the request line and headers, subject to `header_read_timeout`.

### Step 2: NormalizePathLayer

`tower_http::normalize_path::NormalizePathLayer::trim_trailing_slash()` strips
any trailing `/` from the URI path. A request to `GET /get/` becomes `GET /get`.

### Step 3: CorsLayer

`tower_http::cors::CorsLayer::permissive()` adds permissive CORS headers to the
response on the way back out. On inbound, it's a no-op for non-preflight
requests.

### Step 4: CompressionLayer

`tower_http::compression::CompressionLayer` checks the `Accept-Encoding` header.
If the client accepts `gzip` or `br`, it wraps the response body in a
compressed stream on the way out.

### Step 5: TraceLayer

`tower_http::trace::TraceLayer` logs the request (method, path, version) at
`INFO` level on entry, and logs the response status + latency on exit.

### Step 6: timing_middleware

`src/server/timing_layer.rs:15-21`:

```rust
pub async fn timing_middleware(mut request: Request, next: Next) -> Response<Body> {
    request.extensions_mut().insert(RequestTiming::now());
    next.run(request).await
}
```

Creates a `RequestTiming { start: Instant::now() }` and inserts it into the
request's extensions map. Handlers can extract this via
`Option<Extension<RequestTiming>>`.

### Step 7: chaos_middleware (if enabled)

Evaluates three stages: failure, delay, corruption. See
[Section 6.3](#63-chaos-middleware) for the full deep dive.

For a normal request (no chaos triggered), it calls `next.run(request).await`
and returns the response unmodified.

### Step 8: metrics_middleware (if enabled)

`src/server/metrics_layer.rs:15-34`:

```rust
pub async fn metrics_middleware(
    request: Request, next: Next, metrics: Arc<Metrics>,
) -> Response<Body> {
    let path = request.uri().path().to_string();
    let normalized_path = normalize_path(&path);
    let response = next.run(request).await;
    let status = response.status().as_u16();
    metrics.record_request(&normalized_path, status);
    response
}
```

Captures the path *before* calling the handler, then records the status code
*after*. The path `/get` passes through `normalize_path()` unchanged (only
`/status/*`, `/delay/*`, and `/anything/*` get normalized).

### Step 9: Route Handler — `get_handler()`

`src/routes/core_routes.rs:400-407`:

```rust
pub async fn get_handler(
    headers: HeaderMap,
    timing: Option<Extension<RequestTiming>>,
) -> Response {
    let payload = json!({
        "method": "GET",
        "headers": serialize_headers(&headers),
    });
    let duration_ms = timing.map(|t| t.elapsed_ms());
    format_json_response_with_timing(payload, duration_ms)
}
```

1. Axum extracts `headers` (all request headers) and `timing` (the
   `RequestTiming` inserted by the timing middleware).
2. Builds a JSON object with `method` and `headers`.
3. Calculates `duration_ms` from the timing extension.
4. Calls `format_json_response_with_timing()`.

### Step 10: serialize_headers()

`src/routes/core_routes.rs:38-49`:

```rust
fn serialize_headers(headers: &HeaderMap) -> serde_json::Value {
    headers
        .iter()
        .map(|(k, v)| {
            (
                k.to_string(),
                serde_json::Value::String(
                    v.to_str().unwrap_or("<invalid utf8>").to_string()
                ),
            )
        })
        .collect::<serde_json::Map<_, _>>()
        .into()
}
```

Iterates over the `HeaderMap`, converting each header name/value pair into a
JSON key-value entry. Non-UTF-8 values are replaced with `"<invalid utf8>"`.

### Step 11: format_json_response_with_timing()

`src/utils/json_response.rs:35-66`:

```rust
pub fn format_json_response_with_timing(
    mut data: Value, duration_ms: Option<f64>,
) -> Response {
    // If timing is provided and data is an object, inject it
    if let Some(ms) = duration_ms {
        if let Some(obj) = data.as_object_mut() {
            obj.insert("timing".to_string(), json!({ "duration_ms": ms }));
        }
    }

    let body = serde_json::to_string_pretty(&data);

    match body {
        Ok(json_string) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(Body::from(json_string))
            .unwrap_or_else(|_| { /* fallback 500 */ }),
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"error":"Failed to serialize response"}"#))
            .expect("fallback response should always build"),
    }
}
```

1. If `duration_ms` is `Some`, injects a `timing` object into the JSON.
2. Pretty-prints the JSON with `to_string_pretty`.
3. Builds a 200 OK response with `Content-Type: application/json`.
4. Two fallback chains: if `Response::builder()` fails, a simpler 500 response
   is constructed. If serialization itself fails, a hardcoded error string is
   returned.

### Step 12: Response Walks Back Through Middleware

The response travels back up through each middleware layer:

1. **metrics_middleware** records `(path="/get", status=200)`.
2. **chaos_middleware** may add `X-Chaos` header if any chaos was applied.
3. **timing_middleware** is a no-op on the response path.
4. **TraceLayer** logs the response status and elapsed time.
5. **CompressionLayer** compresses the response body if client accepts it.
6. **CorsLayer** adds `Access-Control-*` headers.
7. **NormalizePathLayer** is a no-op on the response path.

### Final Response Example

```json
{
  "method": "GET",
  "headers": {
    "host": "localhost:8080",
    "user-agent": "curl/7.88.1",
    "accept": "*/*"
  },
  "timing": {
    "duration_ms": 0.42
  }
}
```

---

## 5. Route Handlers Reference

### 5.1 Endpoint Summary Table

| # | Path | Method(s) | Handler | Module |
|---|------|-----------|---------|--------|
| 1 | `/` | GET | `root_handler` | `core_routes.rs:379` |
| 2 | `/get` | GET | `get_handler` | `core_routes.rs:400` |
| 3 | `/get` | HEAD | `head_handler` | `core_routes.rs:427` |
| 4 | `/post` | POST | `post_handler` | `core_routes.rs:611` |
| 5 | `/put` | PUT | `put_handler` | `core_routes.rs:654` |
| 6 | `/patch` | PATCH | `patch_handler` | `core_routes.rs:697` |
| 7 | `/delete` | DELETE | `delete_handler` | `core_routes.rs:738` |
| 8 | `/options` | OPTIONS | `options_handler` | `core_routes.rs:787` |
| 9 | `/status/:code` | ANY | `status_handler` | `core_routes.rs:270` |
| 10 | `/anything` | ANY | `anything_handler` | `core_routes.rs:298` |
| 11 | `/anything/*path` | ANY | `anything_handler` | `core_routes.rs:298` |
| 12 | `/uuid` | GET | `uuid_handler` | `core_routes.rs:485` |
| 13 | `/ip` | GET | `ip_handler` | `core_routes.rs:509` |
| 14 | `/user-agent` | GET | `user_agent_handler` | `core_routes.rs:547` |
| 15 | `/headers` | GET | `headers_handler` | `core_routes.rs:579` |
| 16 | `/endpoints` | GET | `endpoints_handler` | `core_routes.rs:454` |
| 17 | `/healthz` | GET | `healthz_handler` | `healthz.rs:21` |
| 18 | `/delay/:n` | ANY | `delay_handler` | `delay.rs:26` |
| 19 | `/redirect/:n` | ANY | `redirect_handler` | `redirect.rs:33` |
| 20 | `/metrics` | GET | `get_metrics` | `metrics.rs:43` |
| 21 | `/swagger-ui` | GET | *(utoipa-swagger-ui)* | `main.rs:141` |
| 22 | `/cookies` | GET | `cookies_handler` | `cookies.rs:60` |
| 23 | `/cookies/set` | GET | `set_cookies_handler` | `cookies.rs:88` |
| 24 | `/cookies/delete` | GET | `delete_cookies_handler` | `cookies.rs:121` |

### 5.2 Echo Handlers

All echo handlers share a common pattern:

1. Extract `headers: HeaderMap` and `timing: Option<Extension<RequestTiming>>`.
2. For body-accepting methods (POST, PUT, PATCH, DELETE), also extract the
   JSON body via `Result<Json<...>, JsonRejection>`.
3. Build a JSON payload with `method`, `headers`, and optionally `body`.
4. Call `format_json_response_with_timing(payload, duration_ms)`.

**`post_handler`** (`src/routes/core_routes.rs:611-628`):

```rust
pub async fn post_handler(
    headers: HeaderMap,
    timing: Option<Extension<RequestTiming>>,
    body: Result<Json<serde_json::Value>, axum::extract::rejection::JsonRejection>,
) -> impl IntoResponse {
    match body {
        Ok(Json(payload_value)) => {
            let response_payload = json!({
                "method": "POST",
                "headers": serialize_headers(&headers),
                "body": payload_value,
            });
            let duration_ms = timing.map(|t| t.elapsed_ms());
            format_json_response_with_timing(response_payload, duration_ms)
        }
        Err(_) => format_error_response(StatusCode::BAD_REQUEST, "Invalid JSON payload"),
    }
}
```

**Differences between echo handlers:**

| Handler | Method string | Body extraction | Body on error |
|---------|--------------|-----------------|---------------|
| `get_handler` | `"GET"` | None | N/A |
| `post_handler` | `"POST"` | `Json<serde_json::Value>` | 400 error |
| `put_handler` | `"PUT"` | `Json<Payload>` (newtype) | 400 error |
| `patch_handler` | `"PATCH"` | `Json<Payload>` (newtype) | 400 error |
| `delete_handler` | `"DELETE"` | `Json<Payload>` (optional) | Returns `body: null` |

Note: `delete_handler` does *not* return a 400 on missing/invalid body. Instead
it echoes `"body": null` — this is intentional since DELETE bodies are optional.

**`anything_handler`** (`src/routes/core_routes.rs:298-320`) is unique: it reads
the raw body bytes via `axum::body::Body` and converts with
`String::from_utf8_lossy`, and also captures the full URI path + query.

### 5.3 Utility Handlers

**`uuid_handler`** (`src/routes/core_routes.rs:485-489`):
Returns `{ "uuid": "<v4 uuid>" }`. Uses `uuid::Uuid::new_v4()`.

**`ip_handler`** (`src/routes/core_routes.rs:509-527`):
IP extraction logic (priority order):
1. `X-Forwarded-For` header — takes the *first* IP from the comma-separated
   list (leftmost = original client).
2. `X-Real-IP` header — fallback for single-hop proxies.
3. `"unknown"` — default when no proxy headers are present.

```rust
let origin = headers
    .get("x-forwarded-for")
    .and_then(|v| v.to_str().ok())
    .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
    .or_else(|| {
        headers
            .get("x-real-ip")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    })
    .unwrap_or_else(|| "unknown".to_string());
```

**`user_agent_handler`** (`src/routes/core_routes.rs:547-559`):
Returns `{ "user-agent": "<value>" }`. Falls back to empty string if header
is missing.

**`headers_handler`** (`src/routes/core_routes.rs:579-585`):
Returns `{ "headers": { ... } }` with all request headers serialized.

### 5.4 Special Handlers

**`status_handler`** (`src/routes/core_routes.rs:270-277`):

```rust
pub async fn status_handler(
    axum::extract::Path(code): axum::extract::Path<u16>,
    _method: axum::http::Method,
) -> Response {
    StatusCode::from_u16(code)
        .unwrap_or(StatusCode::BAD_REQUEST)
        .into_response()
}
```

Accepts any HTTP method. Returns the status code from the path parameter. If the
code is not a valid HTTP status (e.g., 999), defaults to 400 Bad Request.

**`options_handler`** (`src/routes/core_routes.rs:787-796`):
Returns 204 No Content with an `Allow` header listing all supported methods.

**`root_handler`** (`src/routes/core_routes.rs:379-381`):
Returns plain text `"Welcome to Echo Server!\n"`.

**`head_handler`** (`src/routes/core_routes.rs:427-432`):
Returns an empty body with 200 OK status. (Axum automatically strips the body
for HEAD requests, but this handler explicitly returns an empty body.)

**`endpoints_handler`** (`src/routes/core_routes.rs:454-465`):
Serializes the static `API_ENDPOINTS` array into JSON. The array is defined
at `src/routes/core_routes.rs:69-204` and lists all 20 endpoints with their
path, method, and description.

### 5.5 Infrastructure Handlers

**`healthz_handler`** (`src/routes/healthz.rs:21-23`):

```rust
pub async fn healthz_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}
```

Simple health check — returns 200 with plain text "OK".

**`delay_handler`** (`src/routes/delay.rs:26-44`):

```rust
pub async fn delay_handler(
    axum::extract::Path(n): axum::extract::Path<u64>,
    _method: axum::http::Method,
    _body: axum::body::Body,
) -> impl IntoResponse {
    if n > MAX_DELAY_SECONDS {
        return (StatusCode::BAD_REQUEST, format!(
            "Delay of {} seconds exceeds maximum allowed value of {} seconds",
            n, MAX_DELAY_SECONDS
        )).into_response();
    }
    tokio::time::sleep(std::time::Duration::from_secs(n)).await;
    (StatusCode::OK, format!("Response delayed by {} seconds", n)).into_response()
}
```

Caps at `MAX_DELAY_SECONDS` (300) to prevent DoS.

**`redirect_handler`** (`src/routes/redirect.rs:33-56`):

```rust
pub async fn redirect_handler(axum::extract::Path(n): axum::extract::Path<u32>) -> Response {
    if n > MAX_REDIRECT_HOPS {
        return (StatusCode::BAD_REQUEST, format!(
            "Redirect count of {} exceeds maximum allowed value of {}",
            n, MAX_REDIRECT_HOPS
        )).into_response();
    }

    if n == 0 {
        return (StatusCode::OK, "Redirect complete".to_string()).into_response();
    }

    let location = if n == 1 {
        "/get".to_string()
    } else {
        format!("/redirect/{}", n - 1)
    };

    (StatusCode::FOUND, [(header::LOCATION, location)]).into_response()
}
```

Returns a chain of HTTP 302 redirects that decrement `n` on each hop. When `n`
reaches 1, redirects to `/get` as the final destination. When `n` is 0, returns
200 OK directly. Caps at `MAX_REDIRECT_HOPS` (20) to prevent abuse.

**`cookies_handler`** (`src/routes/cookies.rs:60-67`):

```rust
pub async fn cookies_handler(
    headers: HeaderMap,
    timing: Option<Extension<RequestTiming>>,
) -> Response {
    let cookies = parse_cookies(&headers);
    let duration_ms = timing.map(|t| t.elapsed_ms());
    format_json_response_with_timing(json!({"cookies": cookies}), duration_ms)
}
```

Parses the `Cookie` header into a `HashMap<String, String>` using the helper
`parse_cookies()`, which splits on `; ` then on `=`. Returns a JSON object with
a `cookies` key containing all cookie name-value pairs. Supports timing.

**`set_cookies_handler`** (`src/routes/cookies.rs:88-101`):

```rust
pub async fn set_cookies_handler(
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Response {
    let mut response = (StatusCode::FOUND, [(header::LOCATION, "/cookies")]).into_response();
    let response_headers = response.headers_mut();
    for (name, value) in &params {
        if let Ok(cookie_val) = header::HeaderValue::from_str(&format!("{name}={value}; Path=/")) {
            response_headers.append(header::SET_COOKIE, cookie_val);
        }
    }
    response
}
```

Each query parameter becomes a `Set-Cookie` response header with `Path=/`.
Responds with a 302 redirect to `/cookies` so the client can see the result.

**`delete_cookies_handler`** (`src/routes/cookies.rs:121-136`):

```rust
pub async fn delete_cookies_handler(
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Response {
    let mut response = (StatusCode::FOUND, [(header::LOCATION, "/cookies")]).into_response();
    let response_headers = response.headers_mut();
    for name in params.keys() {
        if let Ok(cookie_val) =
            header::HeaderValue::from_str(&format!("{name}=; Max-Age=0; Path=/"))
        {
            response_headers.append(header::SET_COOKIE, cookie_val);
        }
    }
    response
}
```

Each query parameter name generates a `Set-Cookie` header with `Max-Age=0` to
expire the cookie. Like `set_cookies_handler`, redirects to `/cookies`.

**`get_metrics`** (`src/routes/metrics.rs:43-46`):

```rust
pub async fn get_metrics(State(metrics): State<Arc<Metrics>>) -> impl IntoResponse {
    let snapshot = metrics.snapshot();
    (StatusCode::OK, Json(snapshot))
}
```

Uses Axum's `State` extractor to access the shared `Arc<Metrics>`. Returns the
full snapshot as JSON.

---

## 6. Middleware Deep Dives

### 6.1 Timing Middleware

**File:** `src/server/timing_layer.rs` (22 lines total)

```rust
// Full source — src/server/timing_layer.rs:1-22
use axum::{body::Body, extract::Request, middleware::Next, response::Response};
use crate::utils::timing::RequestTiming;

pub async fn timing_middleware(mut request: Request, next: Next) -> Response<Body> {
    request.extensions_mut().insert(RequestTiming::now());
    next.run(request).await
}
```

**`RequestTiming` struct** (`src/utils/timing.rs:13-30`):

```rust
#[derive(Clone, Copy)]
pub struct RequestTiming {
    pub start: Instant,
}

impl RequestTiming {
    pub fn now() -> Self {
        Self { start: Instant::now() }
    }

    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }
}
```

**How handlers access it:**

Handlers declare `timing: Option<Extension<RequestTiming>>`. The `Option`
ensures the handler works even if the timing middleware is somehow bypassed.
When present:

```rust
let duration_ms = timing.map(|t| t.elapsed_ms());
// duration_ms: Option<f64>
```

This is passed to `format_json_response_with_timing()` which injects
`"timing": { "duration_ms": 0.42 }` into the JSON response.

### 6.2 Metrics Middleware

**File:** `src/server/metrics_layer.rs` (55 lines including tests)

```rust
// src/server/metrics_layer.rs:15-34
pub async fn metrics_middleware(
    request: Request, next: Next, metrics: Arc<Metrics>,
) -> Response<Body> {
    let path = request.uri().path().to_string();
    let normalized_path = normalize_path(&path);
    let response = next.run(request).await;
    let status = response.status().as_u16();
    metrics.record_request(&normalized_path, status);
    response
}
```

**Path normalization** (`src/server/metrics_layer.rs:42-55`):

```rust
fn normalize_path(path: &str) -> String {
    let segments: Vec<&str> = path.split('/').collect();
    if segments.len() >= 2 {
        match segments.get(1) {
            Some(&"status") if segments.len() >= 3 => "/status/:code".to_string(),
            Some(&"delay") if segments.len() >= 3  => "/delay/:n".to_string(),
            Some(&"redirect") if segments.len() >= 3 => "/redirect/:n".to_string(),
            Some(&"cookies") if segments.len() >= 3 => {
                let action = segments.get(2).unwrap_or(&"");
                format!("/cookies/{action}")
            }
            Some(&"anything") if segments.len() >= 3 => "/anything/*path".to_string(),
            _ => path.to_string(),
        }
    } else {
        path.to_string()
    }
}
```

Normalization rules:

| Raw path | Normalized | Why |
|----------|-----------|-----|
| `/status/404` | `/status/:code` | Collapse path parameter |
| `/status/500` | `/status/:code` | Same rule |
| `/delay/5` | `/delay/:n` | Collapse path parameter |
| `/redirect/3` | `/redirect/:n` | Collapse path parameter |
| `/cookies/set` | `/cookies/set` | Preserve action segment |
| `/cookies/delete` | `/cookies/delete` | Preserve action segment |
| `/anything/foo/bar` | `/anything/*path` | Collapse wildcard |
| `/get` | `/get` | No change |
| `/anything` | `/anything` | Only 2 segments, no collapse |

**Why normalization matters:** Without it, every unique status code or delay
value would create a separate metrics entry (e.g., `/status/200`,
`/status/404`, `/status/503`...), inflating the metrics HashMap. Normalization
groups them under canonical route names.

### 6.3 Chaos Middleware

**File:** `src/server/chaos_layer.rs` (124 lines)

The most complex middleware. Implements a three-stage chaos injection pipeline.

**Evaluation flow:**

```
         chaos_middleware()
              |
              v
     +------------------+
     | 1. FAILURE ROLL  |
     |  rng < rate?     |
     +--------+---------+
              |
         yes /  \ no
            /    \
           v      v
     [short-circuit    +------------------+
      with error       | 2. DELAY ROLL    |
      response,        |  rng < rate?     |
      RETURN]          +--------+---------+
                                |
                           yes /  \ no
                              /    \
                             v      v
                      [tokio::sleep  |
                       delay_ms]     |
                             \      /
                              \    /
                               v  v
                       +-------------------+
                       | 3. CALL HANDLER   |
                       |   next.run(req)   |
                       +--------+----------+
                                |
                                v
                       +-------------------+
                       | 4. CORRUPTION     |
                       |    ROLL           |
                       |  rng < rate?      |
                       +--------+----------+
                                |
                           yes /  \ no
                              /    \
                             v      v
                    [corrupt body]  |
                             \      /
                              \    /
                               v  v
                       +-------------------+
                       | 5. ADD X-Chaos    |
                       |   HEADER          |
                       |  (if informed &   |
                       |   any applied)    |
                       +--------+----------+
                                |
                                v
                            RESPONSE
```

**Stage 1 — Failure injection** (`src/server/chaos_layer.rs:32-59`):

```rust
if chaos.has_failure() && rng.gen::<f64>() < chaos.failure_rate {
    let code_idx = rng.gen_range(0..chaos.failure_codes.len());
    let status_code = chaos.failure_codes[code_idx];
    applied.push("failure");

    let body = serde_json::json!({
        "error": "Chaos failure injected",
        "chaos": { "type": "failure", "status_code": status_code }
    });

    let mut response = Response::builder()
        .status(StatusCode::from_u16(status_code)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string_pretty(&body).unwrap()))
        .unwrap();

    if chaos.inform_header {
        response.headers_mut()
            .insert("x-chaos", applied.join(",").parse().unwrap());
    }
    return response;
}
```

**Key behavior:** Failure **short-circuits** — the request never reaches the
handler. The response is a JSON error with the randomly selected status code.

**Stage 2 — Delay injection** (`src/server/chaos_layer.rs:62-73`):

```rust
if chaos.has_delay() && rng.gen::<f64>() < chaos.delay_rate {
    let delay_ms = if chaos.delay_ms == "random" {
        rng.gen_range(0..chaos.delay_max_ms)
    } else {
        chaos.delay_ms.parse::<u64>().unwrap_or(0)
    };
    if delay_ms > 0 {
        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
    }
    applied.push("delay");
}
```

The delay happens *before* the handler runs. Two modes:
- **Fixed:** `delay_ms` is parsed as `u64`.
- **Random:** `delay_ms` is `"random"`, and the actual delay is
  `rng.gen_range(0..delay_max_ms)`.

**Stage 3 — Corruption** (`src/server/chaos_layer.rs:79-111`):

After the handler returns, if corruption is enabled and the roll passes:

| `corruption_type` | Behavior |
|-------------------|----------|
| `"empty"` | Replace body with `Body::empty()` |
| `"truncate"` | Read full body into bytes, keep only first half |
| `"garbage"` | Replace each byte with random printable ASCII (0x21-0x7E) |

**Stacking:** Delay and corruption can both apply to the same request. Failure
short-circuits so it never stacks with anything else.

**X-Chaos header:** When `inform_header` is true (default), an `X-Chaos` header
is added listing which chaos types were applied, e.g., `x-chaos: delay,corruption`.

**RNG:** Uses `StdRng::from_entropy()` per request for a fresh, independent
random source.

---

## 7. Configuration System

### 7.1 Config and ChaosConfig Structs

**`Config`** (`src/utils/config.rs:125-160`):

```rust
pub struct Config {
    pub prefix: String,                    // Installation prefix path
    pub log_level: String,                 // "info", "debug", "warn", "error"
    pub server_listen_primary: String,     // e.g., "0.0.0.0:8080"
    pub server_listen_secondary: String,   // e.g., "0.0.0.0:9090"
    pub server_listen_tcp: Option<String>, // e.g., "0.0.0.0:7777"
    pub server_listen_udp: Option<String>, // e.g., "0.0.0.0:7778"
    pub ssl_cert: Option<String>,          // path to PEM cert
    pub ssl_key: Option<String>,           // path to PEM key
    pub metrics_enabled: bool,
    pub compression_enabled: bool,
    pub http_keep_alive_timeout: u64,      // seconds
    pub tcp_keepalive_time: u64,           // seconds
    pub tcp_keepalive_interval: u64,       // seconds
    pub tcp_keepalive_retries: u32,
    pub tcp_nodelay: bool,
    pub header_read_timeout: u64,          // seconds
    pub chaos: ChaosConfig,
}
```

**`ChaosConfig`** (`src/utils/config.rs:17-36`):

```rust
pub struct ChaosConfig {
    pub modes: Vec<String>,         // e.g., ["failure", "delay"]
    pub failure_rate: f64,          // 0.01-1.0
    pub failure_codes: Vec<u16>,    // e.g., [500, 502, 503]
    pub delay_rate: f64,            // 0.01-1.0
    pub delay_ms: String,           // milliseconds or "random"
    pub delay_max_ms: u64,          // max when delay_ms="random"
    pub corruption_rate: f64,       // 0.01-1.0
    pub corruption_type: String,    // "empty", "truncate", "garbage"
    pub inform_header: bool,        // add X-Chaos header (default true)
}
```

### 7.2 Complete Field Reference

| Field | Type | Default | Config Key | Env Var |
|-------|------|---------|-----------|---------|
| `prefix` | `String` | `"/usr/local/rucho"` | `prefix` | `RUCHO_PREFIX` |
| `log_level` | `String` | `"info"` | `log_level` | `RUCHO_LOG_LEVEL` |
| `server_listen_primary` | `String` | `"0.0.0.0:8080"` | `server_listen_primary` | `RUCHO_SERVER_LISTEN_PRIMARY` |
| `server_listen_secondary` | `String` | `"0.0.0.0:9090"` | `server_listen_secondary` | `RUCHO_SERVER_LISTEN_SECONDARY` |
| `server_listen_tcp` | `Option<String>` | `None` | `server_listen_tcp` | `RUCHO_SERVER_LISTEN_TCP` |
| `server_listen_udp` | `Option<String>` | `None` | `server_listen_udp` | `RUCHO_SERVER_LISTEN_UDP` |
| `ssl_cert` | `Option<String>` | `None` | `ssl_cert` | `RUCHO_SSL_CERT` |
| `ssl_key` | `Option<String>` | `None` | `ssl_key` | `RUCHO_SSL_KEY` |
| `metrics_enabled` | `bool` | `false` | `metrics_enabled` | `RUCHO_METRICS_ENABLED` |
| `compression_enabled` | `bool` | `false` | `compression_enabled` | `RUCHO_COMPRESSION_ENABLED` |
| `http_keep_alive_timeout` | `u64` | `75` | `http_keep_alive_timeout` | `RUCHO_HTTP_KEEP_ALIVE_TIMEOUT` |
| `tcp_keepalive_time` | `u64` | `60` | `tcp_keepalive_time` | `RUCHO_TCP_KEEPALIVE_TIME` |
| `tcp_keepalive_interval` | `u64` | `15` | `tcp_keepalive_interval` | `RUCHO_TCP_KEEPALIVE_INTERVAL` |
| `tcp_keepalive_retries` | `u32` | `5` | `tcp_keepalive_retries` | `RUCHO_TCP_KEEPALIVE_RETRIES` |
| `tcp_nodelay` | `bool` | `true` | `tcp_nodelay` | `RUCHO_TCP_NODELAY` |
| `header_read_timeout` | `u64` | `30` | `header_read_timeout` | `RUCHO_HEADER_READ_TIMEOUT` |
| `chaos.modes` | `Vec<String>` | `[]` | `chaos_mode` | `RUCHO_CHAOS_MODE` |
| `chaos.failure_rate` | `f64` | `0.0` | `chaos_failure_rate` | `RUCHO_CHAOS_FAILURE_RATE` |
| `chaos.failure_codes` | `Vec<u16>` | `[]` | `chaos_failure_codes` | `RUCHO_CHAOS_FAILURE_CODES` |
| `chaos.delay_rate` | `f64` | `0.0` | `chaos_delay_rate` | `RUCHO_CHAOS_DELAY_RATE` |
| `chaos.delay_ms` | `String` | `""` | `chaos_delay_ms` | `RUCHO_CHAOS_DELAY_MS` |
| `chaos.delay_max_ms` | `u64` | `0` | `chaos_delay_max_ms` | `RUCHO_CHAOS_DELAY_MAX_MS` |
| `chaos.corruption_rate` | `f64` | `0.0` | `chaos_corruption_rate` | `RUCHO_CHAOS_CORRUPTION_RATE` |
| `chaos.corruption_type` | `String` | `""` | `chaos_corruption_type` | `RUCHO_CHAOS_CORRUPTION_TYPE` |
| `chaos.inform_header` | `bool` | `true` | `chaos_inform_header` | `RUCHO_CHAOS_INFORM_HEADER` |

### 7.3 Loading Precedence

Configuration is loaded in layers, with each layer overriding the previous:

```
1. Hardcoded defaults       Config::default()
        |
        v  (override)
2. /etc/rucho/rucho.conf    system-wide config
        |
        v  (override)
3. ./rucho.conf             local/project config
        |
        v  (override)
4. RUCHO_* env vars         environment variables (highest priority)
```

Implementation: `Config::load_from_paths_with_env()` at `src/utils/config.rs:344-510`.

This method accepts an injectable `env_reader: &dyn Fn(&str) -> Result<String, VarError>`
parameter. Production code passes `env::var`; tests pass a mock HashMap-backed
closure for parallel-safe isolation.

`Config::load_from_paths()` at `src/utils/config.rs:511-515` delegates to
`load_from_paths_with_env()` with `&|key| env::var(key)`.

`Config::load()` at `src/utils/config.rs:684-686` simply calls
`load_from_paths(None, None)` which uses the default paths and real env vars.

### 7.4 The `load_env_var!` Macro

**File:** `src/utils/config.rs:81-111`

The macro accepts an `$env_reader` callable (e.g. `env::var` or a test mock)
so that tests can supply a pure HashMap-backed reader instead of mutating the
process environment. It has five variants for different type conversions:

```rust
macro_rules! load_env_var {
    // Variant 1: String field — direct assignment
    ($config:expr, $field:ident, $env_var:expr, $env_reader:expr) => {
        if let Ok(value) = $env_reader($env_var) {
            $config.$field = value;
        }
    };

    // Variant 2: Option<String> field — wrap in Some
    ($config:expr, $field:ident, $env_var:expr, $env_reader:expr, option) => {
        if let Ok(value) = $env_reader($env_var) {
            $config.$field = Some(value);
        }
    };

    // Variant 3: bool field — "true"/"1" = true, anything else = false
    ($config:expr, $field:ident, $env_var:expr, $env_reader:expr, bool) => {
        if let Ok(value) = $env_reader($env_var) {
            $config.$field = value.eq_ignore_ascii_case("true") || value == "1";
        }
    };

    // Variant 4: u64 field — parse, silently ignore invalid values
    ($config:expr, $field:ident, $env_var:expr, $env_reader:expr, u64) => {
        if let Ok(value) = $env_reader($env_var) {
            if let Ok(v) = value.parse::<u64>() {
                $config.$field = v;
            }
        }
    };

    // Variant 5: u32 field — parse, silently ignore invalid values
    ($config:expr, $field:ident, $env_var:expr, $env_reader:expr, u32) => {
        if let Ok(value) = $env_reader($env_var) {
            if let Ok(v) = value.parse::<u32>() {
                $config.$field = v;
            }
        }
    };
}
```

**Usage examples from `load_from_paths_with_env()`:**

```rust
load_env_var!(config, prefix, "RUCHO_PREFIX", env_reader);                             // String
load_env_var!(config, server_listen_tcp, "RUCHO_SERVER_LISTEN_TCP", env_reader, option); // Option<String>
load_env_var!(config, metrics_enabled, "RUCHO_METRICS_ENABLED", env_reader, bool);      // bool
load_env_var!(config, http_keep_alive_timeout, "RUCHO_HTTP_KEEP_ALIVE_TIMEOUT", env_reader, u64); // u64
load_env_var!(config, tcp_keepalive_retries, "RUCHO_TCP_KEEPALIVE_RETRIES", env_reader, u32);     // u32
```

**Note:** Chaos config fields are loaded manually (not via the macro) because
they are nested under `config.chaos.*` and the macro only supports top-level
fields. They also use `env_reader` directly.

### 7.5 File Parsing

`Config::parse_file_contents()` at `src/utils/config.rs:228-331`:

- Iterates over each line of the file contents.
- Skips lines starting with `#` (comments) and empty lines.
- Splits each line on the first `=` character.
- Matches the left side (key) against known config keys.
- For unknown keys, prints a warning to stderr.

**Config file format:**

```ini
# This is a comment
prefix = /usr/local/rucho
log_level = info
server_listen_primary = 0.0.0.0:8080
metrics_enabled = true
chaos_mode = failure,delay
chaos_failure_codes = 500,502,503
```

**List-type fields** (`chaos_mode`, `chaos_failure_codes`) are comma-separated
and split at parse time:

```rust
"chaos_mode" => {
    config.chaos.modes = value
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
}
```

### 7.6 Validation Pipeline

`Config::validate()` at `src/utils/config.rs:528-539` runs three checks in
sequence:

```
validate()
  |
  +-- Check SSL cert/key pairs
  |     ssl_cert without ssl_key => SslCertWithoutKey
  |     ssl_key without ssl_cert => SslKeyWithoutCert
  |
  +-- validate_connection()
  |     http_keep_alive_timeout == 0 => Connection error
  |     tcp_keepalive_time == 0      => Connection error
  |     tcp_keepalive_interval == 0  => Connection error
  |     tcp_keepalive_retries not in 1..=10 => Connection error
  |     header_read_timeout == 0     => Connection error
  |
  +-- validate_chaos()
        (skipped if chaos.modes is empty)
        Check for unknown chaos types
        If failure mode:
          failure_rate must be 0.01..=1.0
          failure_codes must not be empty
          all codes must be 400..=599
        If delay mode:
          delay_rate must be 0.01..=1.0
          delay_ms must not be empty
          if delay_ms == "random": delay_max_ms must be > 0
          otherwise: delay_ms must parse as u64
        If corruption mode:
          corruption_rate must be 0.01..=1.0
          corruption_type must be "empty", "truncate", or "garbage"
```

**Error types** (`src/utils/config.rs:190-221`):

```rust
pub enum ConfigValidationError {
    SslCertWithoutKey,
    SslKeyWithoutCert,
    Connection(String),
    Chaos(String),
}
```

All variants implement `Display` and `Error`.

---

## 8. Server Orchestration

### 8.1 `run_server()`

**File:** `src/server/mod.rs:24-56`

```rust
pub async fn run_server(config: &Config, app: Router) {
    let handle = Handle::new();
    let shutdown = shutdown::shutdown_signal(handle.clone());

    let mut server_handles: Vec<JoinHandle<Result<(), std::io::Error>>> = Vec::new();

    // Setup HTTP/HTTPS listeners
    http::setup_http_listeners(config, app.clone(), handle.clone(), &mut server_handles).await;

    // Setup TCP listener
    if let Some(tcp_addr_str) = &config.server_listen_tcp {
        tcp::setup_tcp_listener(tcp_addr_str, &mut server_handles).await;
    }

    // Setup UDP listener
    if let Some(udp_addr_str) = &config.server_listen_udp {
        if let Some(socket) = udp::bind_udp_socket(udp_addr_str).await {
            let socket = Arc::new(socket);
            udp::setup_udp_listener(socket, &mut server_handles);
        }
    }

    if !server_handles.is_empty() {
        tracing::info!(
            "{} server(s)/listener(s) started. Waiting for shutdown signal...",
            server_handles.len()
        );
        shutdown.await;
        tracing::info!("Shutdown signal received, all servers and listeners are stopping.");
    } else {
        tracing::warn!("No server or listener instances were configured or able to start.");
    }
}
```

**Key design:**

- `Handle` is shared across all `axum_server` instances — when the shutdown
  signal fires, it triggers graceful shutdown for all HTTP/HTTPS servers.
- Each listener (HTTP, HTTPS, TCP, UDP) is spawned as a separate Tokio task.
  The `JoinHandle`s are collected but never explicitly joined — the function
  blocks on `shutdown.await`.
- TCP and UDP listeners are optional (only started if configured).

### 8.2 HTTP/HTTPS Setup Chain

```
run_server()
  |
  +-- setup_http_listeners()            src/server/http.rs:64-105
        |
        +-- parse_listen_address(primary)     strip " ssl" suffix
        +-- parse_listen_address(secondary)   strip " ssl" suffix
        |
        for each (address, is_ssl):
          |
          +-- if is_ssl:
          |     setup_https_listener()  src/server/http.rs:145-173
          |       +-- try_load_rustls_config()  load TLS certs
          |       +-- axum_server::bind_rustls()
          |       +-- configure_http_builder()
          |       +-- tokio::spawn(server_future)
          |
          +-- else:
                setup_http_listener()   src/server/http.rs:108-142
                  +-- TcpListener::bind()
                  +-- listener.into_std()       convert to std for socket2
                  +-- configure_tcp_socket()    set keepalive + nodelay
                  +-- axum_server::Server::from_tcp()
                  +-- configure_http_builder()
                  +-- tokio::spawn(server_future)
```

**Note:** For HTTP listeners, the flow is:
`tokio::net::TcpListener::bind()` -> `into_std()` -> `configure_tcp_socket()`
-> `axum_server::Server::from_tcp()`. The conversion to `std::net::TcpListener`
is necessary because `socket2::SockRef` requires a standard library socket.

### 8.3 TCP Socket Configuration

`configure_tcp_socket()` at `src/server/http.rs:18-36`:

```rust
fn configure_tcp_socket(listener: &std::net::TcpListener, config: &Config) {
    let sock_ref = SockRef::from(listener);

    let keepalive = TcpKeepalive::new()
        .with_time(Duration::from_secs(config.tcp_keepalive_time))
        .with_interval(Duration::from_secs(config.tcp_keepalive_interval));

    // with_retries is not available on Windows
    #[cfg(not(target_os = "windows"))]
    let keepalive = keepalive.with_retries(config.tcp_keepalive_retries);

    if let Err(e) = sock_ref.set_tcp_keepalive(&keepalive) {
        tracing::warn!("Failed to set TCP keep-alive: {}", e);
    }

    if let Err(e) = sock_ref.set_nodelay(config.tcp_nodelay) {
        tracing::warn!("Failed to set TCP_NODELAY: {}", e);
    }
}
```

**Platform gate:** `with_retries()` is not available on Windows. The
`#[cfg(not(target_os = "windows"))]` attribute conditionally compiles this call
only on non-Windows targets. This means on Windows, the retry count is left at
the OS default.

**socket2 usage:** The `SockRef::from(listener)` creates a reference to the
underlying socket file descriptor without taking ownership. This allows setting
socket options on an already-bound listener.

### 8.4 HTTP Builder Configuration

`configure_http_builder()` at `src/server/http.rs:42-58`:

```rust
fn configure_http_builder<A>(server: &mut axum_server::Server<A>, config: &Config) {
    let http_timeout = Duration::from_secs(config.http_keep_alive_timeout);
    let header_timeout = Duration::from_secs(config.header_read_timeout);

    server.http_builder()
        .http1()
        .keep_alive(true)
        .timer(TokioTimer::new())
        .header_read_timeout(header_timeout);

    server.http_builder()
        .http2()
        .keep_alive_interval(Some(http_timeout))
        .keep_alive_timeout(Duration::from_secs(20));
}
```

**HTTP/1.1 settings:**
- `keep_alive(true)` — enable HTTP/1.1 keep-alive connections.
- `timer(TokioTimer::new())` — required by hyper for timeout tracking.
- `header_read_timeout(header_timeout)` — abort if headers aren't received
  within this duration.

**HTTP/2 settings:**
- `keep_alive_interval` — send PING frames at this interval to detect dead
  connections.
- `keep_alive_timeout(20s)` — close connection if PING isn't acknowledged
  within 20 seconds.

### 8.5 TLS Configuration

**`parse_listen_address()`** (`src/utils/server_config.rs:92-106`):

```rust
pub fn parse_listen_address(listen_str: &str) -> Option<(String, bool)> {
    if listen_str.is_empty() { return None; }
    let lower = listen_str.to_lowercase();
    if lower.ends_with(" ssl") {
        let addr = listen_str[..listen_str.len() - 4].to_string();
        Some((addr, true))
    } else {
        Some((listen_str.to_string(), false))
    }
}
```

Input/output examples:
- `"0.0.0.0:8080"` -> `Some(("0.0.0.0:8080", false))`
- `"0.0.0.0:443 ssl"` -> `Some(("0.0.0.0:443", true))`
- `"0.0.0.0:443 SSL"` -> `Some(("0.0.0.0:443", true))`
- `""` -> `None`

**`try_load_rustls_config()`** (`src/utils/server_config.rs:27-64`):

```rust
pub async fn try_load_rustls_config(
    ssl_cert_path_opt: Option<&str>,
    ssl_key_path_opt: Option<&str>,
) -> Option<RustlsConfig> {
    let (cert_p, key_p) = match (ssl_cert_path_opt, ssl_key_path_opt) {
        (Some(cert_path_str), Some(key_path_str)) => (cert_path_str, key_path_str),
        _ => { return None; }
    };

    let cert_path = PathBuf::from(cert_p);
    let key_path = PathBuf::from(key_p);

    if cert_path.exists() && key_path.exists() {
        match RustlsConfig::from_pem_file(&cert_path, &key_path).await {
            Ok(config) => Some(config),
            Err(err) => { tracing::error!("..."); None }
        }
    } else {
        tracing::warn!("..."); None
    }
}
```

Returns `None` in three cases:
1. Either path is `None`.
2. Files don't exist on disk.
3. `RustlsConfig::from_pem_file()` fails (invalid PEM, etc.).

---

## 9. TCP and UDP Echo Handlers

### 9.1 TCP Echo Loop

**File:** `src/tcp_udp_handlers.rs:38-74`

```rust
pub async fn handle_tcp_connection(mut stream: TcpStream) {
    let peer_addr = match stream.peer_addr() {
        Ok(addr) => addr.to_string(),
        Err(_) => "unknown peer".to_string(),
    };
    tracing::info!("Accepted TCP connection from: {}", peer_addr);

    let mut buf = vec![0u8; MAX_BUFFER_SIZE.min(65536)];

    loop {
        match stream.read(&mut buf).await {
            Ok(0) => {
                tracing::info!("TCP connection closed by client: {}", peer_addr);
                break;
            }
            Ok(n) => {
                tracing::info!("Received {} bytes from {}: {:?}", n, peer_addr,
                    String::from_utf8_lossy(&buf[..n]));
                if let Err(e) = stream.write_all(&buf[..n]).await {
                    tracing::error!("Failed to write to TCP stream for {}: {}", peer_addr, e);
                    break;
                }
                tracing::info!("Echoed {} bytes back to {}", n, peer_addr);
            }
            Err(e) => {
                tracing::error!("Failed to read from TCP stream for {}: {}", peer_addr, e);
                break;
            }
        }
    }
}
```

**Lifecycle:**

```
[client connects]
      |
      v
handle_tcp_connection(stream)
      |
      +-- get peer_addr
      +-- allocate buffer (min(MAX_BUFFER_SIZE, 65536) = 65536 bytes)
      |
      +-- LOOP:
            |
            +-- stream.read(&mut buf)
                  |
                  +-- Ok(0)  => client closed => break
                  +-- Ok(n)  => write_all(buf[..n]) => echo back
                  |             write error => break
                  +-- Err(e) => read error => break
```

**Security:** Buffer is capped at 65536 bytes (`MAX_BUFFER_SIZE`). Each
`read()` call returns at most `buf.len()` bytes, so memory usage per
connection is bounded.

### 9.2 TCP Listener Setup

**File:** `src/server/tcp.rs:12-51`

```rust
pub async fn setup_tcp_listener(
    tcp_addr_str: &str,
    server_handles: &mut Vec<JoinHandle<Result<(), std::io::Error>>>,
) {
    let addr: std::net::SocketAddr = match tcp_addr_str.parse() {
        Ok(addr) => addr,
        Err(e) => { tracing::error!("..."); return; }
    };

    match TcpListener::bind(addr).await {
        Ok(listener) => {
            tracing::info!("Starting TCP echo listener on {}", addr);
            let tcp_listener_handle = tokio::spawn(async move {
                loop {
                    match listener.accept().await {
                        Ok((socket, client_addr)) => {
                            tracing::info!("Accepted new TCP connection from {}", client_addr);
                            tokio::spawn(handle_tcp_connection(socket));
                        }
                        Err(e) => {
                            tracing::error!("Failed to accept TCP connection: {}", e);
                        }
                    }
                }
                #[allow(unreachable_code)]
                Ok::<(), std::io::Error>(())
            });
            server_handles.push(tcp_listener_handle);
        }
        Err(e) => { tracing::error!("..."); }
    }
}
```

**Design:** Each accepted connection spawns a new Tokio task running
`handle_tcp_connection`. The accept loop runs indefinitely — accept errors are
logged but don't stop the listener.

### 9.3 UDP Echo with Exponential Backoff

**File:** `src/tcp_udp_handlers.rs:95-158`

```rust
pub async fn handle_udp_socket(socket: Arc<UdpSocket>) -> std::io::Result<()> {
    let mut buf = vec![0u8; MAX_BUFFER_SIZE.min(65536)];
    let mut consecutive_errors: u32 = 0;

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((size, src_addr)) => {
                consecutive_errors = 0;  // reset on success
                // ... log and echo back ...
                if let Err(e) = socket.send_to(&buf[..size], src_addr).await {
                    tracing::error!("...");
                }
            }
            Err(e) => {
                tracing::error!("...");
                consecutive_errors = consecutive_errors.saturating_add(1);
                let backoff_ms = UDP_ERROR_BACKOFF_BASE_MS
                    .saturating_mul(2u64.saturating_pow(consecutive_errors.min(10)))
                    .min(UDP_ERROR_BACKOFF_MAX_MS);
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
            }
        }
    }
}
```

**Backoff formula:**

```
backoff_ms = min(100 * 2^consecutive_errors, 5000) ms
```

Where:
- `UDP_ERROR_BACKOFF_BASE_MS` = 100
- `UDP_ERROR_BACKOFF_MAX_MS` = 5000
- `consecutive_errors` is capped at 10 for the exponent (via `.min(10)`)

| Consecutive errors | Backoff (ms) |
|-------------------|-------------|
| 1 | 200 |
| 2 | 400 |
| 3 | 800 |
| 4 | 1600 |
| 5 | 3200 |
| 6+ | 5000 (capped) |

On any successful `recv_from`, the counter resets to 0.

**Why backoff matters:** Without it, a persistent error (e.g., socket closed by
OS) would cause a hot loop consuming 100% CPU. The backoff ensures the loop
slows down under error conditions.

### 9.4 UDP Listener Setup

**File:** `src/server/udp.rs:18-54`

Two functions:

1. `bind_udp_socket(udp_addr_str)` — parses address and binds a `UdpSocket`.
   Returns `Option<UdpSocket>`.

2. `setup_udp_listener(socket, server_handles)` — wraps the socket in `Arc`
   and spawns `handle_udp_socket(socket)` as a Tokio task.

Unlike TCP, UDP doesn't have per-connection tasks — a single task handles all
datagrams on the socket.

---

## 10. Metrics Collection and Rolling Window

### 10.1 Data Model

**File:** `src/utils/metrics.rs`

```rust
// src/utils/metrics.rs:66-79
pub struct Metrics {
    total_requests: AtomicU64,                       // all-time request count
    total_successes: AtomicU64,                      // all-time 2xx count
    total_failures: AtomicU64,                       // all-time 4xx/5xx count
    endpoint_hits: RwLock<HashMap<String, u64>>,     // all-time per-endpoint
    rolling_buckets: RwLock<Vec<TimeBucket>>,         // 60 one-minute buckets
    current_bucket_idx: RwLock<usize>,               // index of active bucket
}
```

**Thread safety:**
- `AtomicU64` for counters — lock-free, uses `Ordering::Relaxed` (sufficient
  since we only need eventual consistency for metrics).
- `RwLock<HashMap>` for endpoint hits — allows concurrent readers, exclusive
  writer.
- `RwLock<Vec<TimeBucket>>` for rolling window — same semantics.

### 10.2 TimeBucket Struct

```rust
// src/utils/metrics.rs:22-33
struct TimeBucket {
    start_time: Option<Instant>,              // None if never used
    requests: u64,
    successes: u64,
    failures: u64,
    endpoint_hits: HashMap<String, u64>,
}
```

**Methods:**

| Method | Description |
|--------|-------------|
| `reset(start_time)` | Clear all counters, set new start time |
| `is_expired(now)` | True if `now - start_time >= 60 seconds` |
| `is_within_window(now, window)` | True if `now - start_time < window` |

**Constants:**
- `ROLLING_WINDOW_BUCKETS` = 60 (one per minute)
- `BUCKET_DURATION` = 60 seconds

### 10.3 Recording Flow

`Metrics::record_request(endpoint, status_code)` at `src/utils/metrics.rs:109-129`:

```
record_request(endpoint, status_code)
  |
  +-- classify: is_success = (200..300), is_failure = (>= 400)
  |
  +-- Atomic increment:
  |     total_requests += 1
  |     if is_success: total_successes += 1
  |     if is_failure: total_failures += 1
  |
  +-- Update all-time endpoint_hits:
  |     lock write -> entry(endpoint).or_insert(0) += 1
  |
  +-- update_rolling_window(now, endpoint, is_success, is_failure)
        |
        +-- lock write on rolling_buckets + current_bucket_idx
        +-- if current bucket is expired:
        |     advance index: (idx + 1) % 60
        |     reset new current bucket with now
        +-- increment current bucket:
              bucket.requests += 1
              if is_success: bucket.successes += 1
              if is_failure: bucket.failures += 1
              bucket.endpoint_hits[endpoint] += 1
```

**Note:** Status codes 300-399 (redirects) increment `total_requests` but
neither `successes` nor `failures` — they are tracked but not classified.

### 10.4 Querying Flow

`Metrics::snapshot()` at `src/utils/metrics.rs:230-245`:

```
snapshot()
  |
  +-- AllTimeMetrics:
  |     total_requests = total_requests.load(Relaxed)
  |     successes      = total_successes.load(Relaxed)
  |     failures       = total_failures.load(Relaxed)
  |     endpoint_hits  = endpoint_hits.read().clone()
  |
  +-- LastHourMetrics:
        total_requests = sum_rolling_window(|b| b.requests)
        successes      = sum_rolling_window(|b| b.successes)
        failures       = sum_rolling_window(|b| b.failures)
        endpoint_hits  = get_last_hour_endpoint_hits()
```

`sum_rolling_window()` at `src/utils/metrics.rs:214-227`:

```rust
fn sum_rolling_window<F>(&self, extractor: F) -> u64
where F: Fn(&TimeBucket) -> u64 {
    let now = Instant::now();
    let window = Duration::from_secs(3600);
    let buckets = self.rolling_buckets.read().unwrap();
    buckets.iter()
        .filter(|b| b.is_within_window(now, window))
        .map(&extractor)
        .sum()
}
```

Iterates all 60 buckets, keeping only those within the 1-hour window, then
sums the extracted field.

### 10.5 Snapshot Structs

```rust
// src/utils/metrics.rs:249-281
#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricsSnapshot {
    pub all_time: AllTimeMetrics,
    pub last_hour: LastHourMetrics,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AllTimeMetrics {
    pub total_requests: u64,
    pub successes: u64,
    pub failures: u64,
    pub endpoint_hits: HashMap<String, u64>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LastHourMetrics {
    pub total_requests: u64,
    pub successes: u64,
    pub failures: u64,
    pub endpoint_hits: HashMap<String, u64>,
}
```

All three structs derive `serde::Serialize`, so the `/metrics` handler can
return `Json(snapshot)` directly.

---

## 11. Process Management (PID Lifecycle)

**File:** `src/utils/pid.rs`

### PID File Location

The PID file path is hardcoded as a constant:

```rust
// src/utils/constants.rs:19
pub const PID_FILE_PATH: &str = "/var/run/rucho/rucho.pid";
```

### Operations

| Function | Description | Source |
|----------|------------|--------|
| `write_pid_file(pid)` | Creates file, writes PID + newline | `pid.rs:57-61` |
| `read_pid_file()` | Reads file, parses as `usize` | `pid.rs:69-75` |
| `remove_pid_file()` | Deletes the PID file | `pid.rs:82-84` |
| `check_process_running(pid)` | Uses `sysinfo` to check if PID exists | `pid.rs:95-99` |
| `stop_process(pid)` | Sends SIGTERM, waits 1s, checks if stopped | `pid.rs:123-152` |
| `pid_file_path()` | Returns `PID_FILE_PATH` | `pid.rs:155-157` |

### Error Types

```rust
pub enum PidError {
    CreateFailed(std::io::Error),
    WriteFailed(std::io::Error),
    ReadFailed(std::io::Error),
    RemoveFailed(std::io::Error),
    InvalidFormat,
    ProcessNotFound(usize),
    SignalFailed(usize),
}
```

### Stop Result

```rust
pub enum StopResult {
    Stopped,     // Process confirmed dead after SIGTERM
    SignalSent,  // SIGTERM sent but process still running after 1s
    NotFound,    // Process doesn't exist (already stopped)
    Failed,      // Could not send signal
}
```

### `stop_process()` Flow

```
stop_process(pid)
  |
  +-- System::new_all(), refresh_processes()
  +-- system.process(pid)?
        |
        None => StopResult::NotFound
        |
        Some(process) =>
          +-- process.kill_with(Signal::Term)?
                |
                Some(true) =>
                  +-- sleep(1 second)
                  +-- refresh_processes()
                  +-- process still exists?
                        yes => StopResult::SignalSent
                        no  => StopResult::Stopped
                |
                Some(false) | None =>
                  +-- refresh_processes()
                  +-- process still exists?
                        yes => StopResult::Failed
                        no  => StopResult::NotFound
```

### CLI Command Handlers

**`handle_start_command()`** (`src/cli/commands.rs:38-52`):
1. Gets current PID via `process::id()`.
2. Writes PID file.
3. Returns `true` on success (caller proceeds to start server), `false` on
   failure (caller exits without starting).

**`handle_stop_command()`** (`src/cli/commands.rs:55-104`):
1. Reads PID from file.
2. Calls `stop_process(pid)`.
3. On `Stopped` or `NotFound`: removes PID file.
4. On `SignalSent`: warns user may need `kill -9`.
5. If PID file doesn't exist: reports "Server not running".

**`handle_status_command()`** (`src/cli/commands.rs:107-135`):
1. Reads PID from file.
2. Calls `check_process_running(pid)`.
3. Reports running/stopped status.
4. If PID file exists but process isn't running: suggests cleanup.

**`handle_version_command()`** (`src/cli/commands.rs:138-140`):
Prints `rucho 1.0.0` using `env!("CARGO_PKG_NAME")` and
`env!("CARGO_PKG_VERSION")`.

---

## 12. Response Formatting

### `format_json_response()`

**File:** `src/utils/json_response.rs:17-19`

```rust
pub fn format_json_response(data: Value) -> Response {
    format_json_response_with_timing(data, None)
}
```

Convenience wrapper — delegates to the timing variant with `None`.

### `format_json_response_with_timing()`

**File:** `src/utils/json_response.rs:35-66`

Detailed in [Section 4, Step 11](#step-11-format_json_response_with_timing).

**Fallback chain:**

```
1. Try to inject timing into JSON object
2. Try to pretty-print JSON
   |
   +-- Success: build 200 OK response
   |     |
   |     +-- Response::builder() fails? => 500 "Failed to build response"
   |
   +-- Failure: build 500 "Failed to serialize response"
```

### `format_error_response()`

**File:** `src/utils/error_response.rs:19-39`

```rust
pub fn format_error_response(status: StatusCode, message: &str) -> Response {
    let error_body = json!({ "error": message });

    let body_string = serde_json::to_string(&error_body)
        .unwrap_or_else(|_| format!(r#"{{"error":"{}"}}"#, message.replace('"', "\\\"")));

    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Body::from(body_string))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(r#"{"error":"Failed to build error response"}"#))
                .expect("fallback response should always build")
        })
}
```

**Fallback chain:**

```
1. Try serde_json::to_string
   |
   +-- Success: use serialized JSON
   +-- Failure: use format!() with manual escaping
        |
        v
2. Try Response::builder()
   |
   +-- Success: return response
   +-- Failure: build hardcoded 500 fallback
```

### `serialize_headers()`

**File:** `src/routes/core_routes.rs:38-49`

Detailed in [Section 4, Step 10](#step-10-serialize_headers). Converts a
`HeaderMap` into a `serde_json::Value` JSON object. Non-UTF-8 header values
become `"<invalid utf8>"`.

---

## 13. Graceful Shutdown

**File:** `src/server/shutdown.rs` (17 lines total)

```rust
pub async fn shutdown_signal(handle: Handle) {
    signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    tracing::info!("Signal received, starting graceful shutdown");
    handle.graceful_shutdown(Some(Duration::from_secs(5)));
}
```

**Behavior:**

1. `signal::ctrl_c().await` — blocks until Ctrl+C is received.
2. Calls `handle.graceful_shutdown(Some(Duration::from_secs(5)))` on the
   shared `axum_server::Handle`.
3. This tells all HTTP/HTTPS servers sharing this handle to:
   - Stop accepting new connections.
   - Wait up to 5 seconds for in-flight requests to complete.
   - Force-close any remaining connections after 5 seconds.

**Note:** The TCP and UDP echo listeners are *not* gracefully shut down —
they run in spawned tasks that will be dropped when the Tokio runtime shuts
down. Since they're stateless echo handlers, this is acceptable.

---

## 14. OpenAPI / Swagger Integration

**File:** `src/main.rs:33-71`

```rust
#[derive(OpenApi)]
#[openapi(
    paths(
        rucho::routes::core_routes::root_handler,
        rucho::routes::core_routes::get_handler,
        rucho::routes::core_routes::head_handler,
        rucho::routes::core_routes::post_handler,
        rucho::routes::core_routes::put_handler,
        rucho::routes::core_routes::patch_handler,
        rucho::routes::core_routes::delete_handler,
        rucho::routes::core_routes::options_handler,
        rucho::routes::core_routes::status_handler,
        rucho::routes::core_routes::anything_handler,
        rucho::routes::core_routes::anything_path_handler,
        rucho::routes::core_routes::endpoints_handler,
        rucho::routes::delay::delay_handler,
        rucho::routes::healthz::healthz_handler,
        rucho::routes::redirect::redirect_handler,
        rucho::routes::cookies::cookies_handler,
        rucho::routes::cookies::set_cookies_handler,
        rucho::routes::cookies::delete_cookies_handler,
        rucho::routes::core_routes::uuid_handler,
        rucho::routes::core_routes::ip_handler,
        rucho::routes::core_routes::user_agent_handler,
        rucho::routes::core_routes::headers_handler,
    ),
    components(
        schemas(EndpointInfo, rucho::routes::core_routes::Payload)
    ),
    tags(
        (name = "Rucho", description = "Rucho API")
    )
)]
struct ApiDoc;
```

**How it works:**

1. The `#[derive(OpenApi)]` macro on `ApiDoc` generates an `openapi()` method
   that returns the full OpenAPI specification as a `utoipa::openapi::OpenApi`
   struct.
2. Each handler listed under `paths(...)` must have `#[utoipa::path(...)]`
   annotations.
3. The `components(schemas(...))` section registers reusable schema types.
4. The `tags(...)` section defines API grouping for the Swagger UI.

**Router mount** (`src/main.rs:141`):

```rust
.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
```

This serves:
- `/swagger-ui` — the interactive Swagger UI
- `/api-docs/openapi.json` — the raw OpenAPI JSON spec

**`anything_path_handler` note:** This handler exists *solely* for OpenAPI
documentation. The actual `/anything/*path` requests are handled by
`anything_handler`. The path handler returns 501 if ever called directly
(`src/routes/core_routes.rs:348-362`).

---

## 15. Constants Reference

**File:** `src/utils/constants.rs`

| Constant | Type | Value | Used By |
|----------|------|-------|---------|
| `DEFAULT_PREFIX` | `&str` | `"/usr/local/rucho"` | `Config::default()` |
| `DEFAULT_LOG_LEVEL` | `&str` | `"info"` | `Config::default()` |
| `DEFAULT_SERVER_LISTEN_PRIMARY` | `&str` | `"0.0.0.0:8080"` | `Config::default()` |
| `DEFAULT_SERVER_LISTEN_SECONDARY` | `&str` | `"0.0.0.0:9090"` | `Config::default()` |
| `PID_FILE_PATH` | `&str` | `"/var/run/rucho/rucho.pid"` | `pid.rs` |
| `MAX_DELAY_SECONDS` | `u64` | `300` | `delay_handler` |
| `MAX_REDIRECT_HOPS` | `u32` | `20` | `redirect_handler` |
| `MAX_BUFFER_SIZE` | `usize` | `65536` | `tcp_udp_handlers.rs` |
| `UDP_ERROR_BACKOFF_BASE_MS` | `u64` | `100` | `handle_udp_socket` |
| `UDP_ERROR_BACKOFF_MAX_MS` | `u64` | `5000` | `handle_udp_socket` |
| `DEFAULT_HTTP_KEEP_ALIVE_TIMEOUT_SECS` | `u64` | `75` | `Config::default()` |
| `DEFAULT_TCP_KEEPALIVE_SECS` | `u64` | `60` | `Config::default()` |
| `DEFAULT_TCP_KEEPALIVE_INTERVAL_SECS` | `u64` | `15` | `Config::default()` |
| `DEFAULT_TCP_KEEPALIVE_RETRIES` | `u32` | `5` | `Config::default()` |
| `DEFAULT_HEADER_READ_TIMEOUT_SECS` | `u64` | `30` | `Config::default()` |

---

## 16. Dependency Map

Key external crates and their role in the application:

| Crate | Version | What It Provides |
|-------|---------|-----------------|
| `axum` | 0.7 | HTTP framework — Router, handlers, extractors, middleware |
| `tokio` | 1 (full) | Async runtime — task spawning, I/O, timers, signals |
| `hyper` | 1.0 | HTTP/1.1 and HTTP/2 protocol implementation (under axum) |
| `hyper-util` | 0.1 | `TokioTimer` for hyper's timeout system |
| `tower` | 0.5 | Middleware/service abstraction (tower::Layer, tower::Service) |
| `tower-http` | 0.5 | Trace, CORS, NormalizePath, Compression middleware layers |
| `axum-server` | 0.7 | TLS-capable HTTP server with graceful shutdown `Handle` |
| `clap` | 4.4 | CLI argument parsing with derive macros |
| `serde` | 1.0 | Serialization/deserialization framework |
| `serde_json` | 1.0 | JSON serialization, `json!()` macro, `Value` type |
| `tracing` | 0.1 | Structured logging facade |
| `tracing-subscriber` | 0.3 | Logging output layer (console formatting) |
| `rustls` | 0.23 | Modern TLS library (replaces OpenSSL) |
| `tokio-rustls` | 0.25 | Tokio integration for rustls |
| `rustls-pemfile` | 1.0 | PEM file parsing for certificates and keys |
| `socket2` | 0.5 | Low-level socket options (keepalive, nodelay) via `SockRef` |
| `utoipa` | 4 | OpenAPI spec generation from code annotations |
| `utoipa-swagger-ui` | 7 | Swagger UI serving as an axum route |
| `uuid` | 1 (v4) | UUID v4 generation for `/uuid` endpoint |
| `rand` | 0.8 | Random number generation for chaos middleware |
| `sysinfo` | 0.30 | Process inspection for PID management (`kill`, `process`) |
| `http` | 1.0 | HTTP types (`StatusCode`, `HeaderMap`, etc.) |
| `tempfile` | 3.8 | *(dev only)* Temporary directories for config tests |
| `criterion` | 0.5 | *(dev only)* Benchmark framework with async tokio support and HTML reports |
| `reqwest` | 0.12 | *(dev only)* HTTP client for integration tests (cookie jar, JSON support) |

---

## 17. Source File Index

Complete listing of all source files with line counts and primary purpose:

| File | Purpose |
|------|---------|
| `src/main.rs` | Entry point, `build_app()`, `ApiDoc`, CLI dispatch |
| `src/lib.rs` | Crate root, module declarations |
| `src/cli/mod.rs` | CLI module re-exports |
| `src/cli/commands.rs` | `Args`, `CliCommand`, start/stop/status/version handlers |
| `src/routes/mod.rs` | Routes module re-exports |
| `src/routes/cookies.rs` | `/cookies`, `/cookies/set`, `/cookies/delete` handlers and router |
| `src/routes/core_routes.rs` | 16 route handlers, `router()`, `EndpointInfo`, `API_ENDPOINTS` |
| `src/routes/delay.rs` | `/delay/:n` handler and router |
| `src/routes/healthz.rs` | `/healthz` handler and router |
| `src/routes/metrics.rs` | `/metrics` handler (stateful, `State<Arc<Metrics>>`) |
| `src/routes/redirect.rs` | `/redirect/:n` handler and router |
| `src/server/mod.rs` | `run_server()` — top-level orchestrator |
| `src/server/http.rs` | HTTP/HTTPS listener setup, TCP socket config, HTTP builder config |
| `src/server/tcp.rs` | TCP echo listener setup (accept loop) |
| `src/server/udp.rs` | UDP socket binding and listener setup |
| `src/server/shutdown.rs` | `shutdown_signal()` — Ctrl+C with 5s grace period |
| `src/server/chaos_layer.rs` | Chaos engineering middleware (failure/delay/corruption) |
| `src/server/metrics_layer.rs` | Metrics recording middleware + path normalization |
| `src/server/timing_layer.rs` | Request timing middleware |
| `src/tcp_udp_handlers.rs` | TCP echo loop, UDP echo with exponential backoff |
| `src/utils/mod.rs` | Utils module re-exports |
| `src/utils/config.rs` | `Config`, `ChaosConfig`, loading, validation, `load_env_var!` |
| `src/utils/constants.rs` | All hardcoded constants (15 values) |
| `src/utils/error_response.rs` | `format_error_response()` |
| `src/utils/json_response.rs` | `format_json_response()`, `format_json_response_with_timing()` |
| `src/utils/metrics.rs` | `Metrics`, `TimeBucket`, rolling window, snapshot structs |
| `src/utils/pid.rs` | PID file operations, process management |
| `src/utils/server_config.rs` | `try_load_rustls_config()`, `parse_listen_address()` |
| `src/utils/timing.rs` | `RequestTiming` struct |
| `benches/response_benchmarks.rs` | Criterion microbenchmarks for response building functions |
| `benches/endpoint_benchmarks.rs` | Criterion async benchmarks for full endpoint request cycles via `tower::oneshot` |
| `tests/integration.rs` | Integration tests — real HTTP server per test via `reqwest` (12 tests) |
| `debian/man/rucho.1` | Man page (roff format) — installed to `/usr/share/man/man1/` via `.deb` |

---

## 18. Benchmark Suite

Rucho includes a Criterion-based benchmark suite (`cargo bench`) that establishes
performance baselines for the hot paths.

### Response Building Microbenchmarks (`benches/response_benchmarks.rs`)

Directly benchmarks the public response formatting functions with no router or
middleware overhead:

| Benchmark | Function Under Test | Payload |
|-----------|-------------------|---------|
| `format_json_response (small)` | `format_json_response()` | `{"message": "hello"}` |
| `format_json_response (medium)` | `format_json_response()` | ~10 fields with nested headers object |
| `format_json_response_with_timing (medium)` | `format_json_response_with_timing()` | Medium payload + timing injection |
| `format_error_response` | `format_error_response()` | `404 Not Found` error |

### Endpoint Request Cycle Benchmarks (`benches/endpoint_benchmarks.rs`)

Sends `http::Request` objects through the full Axum router stack using
`tower::ServiceExt::oneshot`. This measures handler + middleware + serialization
without TCP/network overhead.

The benchmark router is built from the library's public route builders with only
the timing middleware layer (no chaos, metrics, tracing, or compression) to
isolate handler performance.

| Benchmark | Method | Path | Notes |
|-----------|--------|------|-------|
| `GET /healthz` | GET | `/healthz` | Minimal response — baseline for router overhead |
| `GET /get` | GET | `/get` | Headers echo + JSON serialization |
| `GET /uuid` | GET | `/uuid` | UUID generation + JSON serialization |
| `POST /post` | POST | `/post` | JSON body parsing + echo + serialization |
| `GET /endpoints` | GET | `/endpoints` | Static list serialization (larger payload) |

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run a specific benchmark file
cargo bench --bench response_benchmarks
cargo bench --bench endpoint_benchmarks

# Quick mode (fewer iterations, faster feedback)
cargo bench -- --quick
```

HTML reports are generated in `target/criterion/`.
