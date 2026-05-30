# API Reference

Complete reference for all Rucho HTTP endpoints.

**Interactive docs:** Start the server and visit [`/swagger-ui`](http://localhost:8080/swagger-ui) for the live OpenAPI explorer, or fetch the raw spec at [`/api-docs/openapi.json`](http://localhost:8080/api-docs/openapi.json).

---

## Table of Contents

- [Echo Endpoints](#echo-endpoints)
- [Utility Endpoints](#utility-endpoints)
- [Status Codes](#status-codes)
- [Wildcard](#wildcard)
- [Redirects](#redirects)
- [Delay](#delay)
- [Cookies](#cookies)
- [Data Formats](#data-formats)
- [Infrastructure](#infrastructure)
- [Documentation](#documentation)

---

## Echo Endpoints

Core echo handlers that reflect request details back to the caller. All echo responses include an `http_version` field and a `timing` object with processing duration. Over **HTTPS**, `/get` and `/anything` additionally include a `tls` object describing the negotiated connection (see [GET /get](#get-get)).

### GET /get

Echo request details (method, headers, timing).

**Response:** `200 OK`

| Field | Type | Description |
|-------|------|-------------|
| `method` | string | `"GET"` |
| `http_version` | string | HTTP version (e.g. `"HTTP/1.1"`, `"HTTP/2.0"`) |
| `headers` | object | All request headers as key-value pairs |
| `timing.duration_ms` | number | Processing time in milliseconds |
| `tls` | object | **HTTPS only** — negotiated TLS connection info (omitted on plain HTTP) |

```json
{
  "method": "GET",
  "http_version": "HTTP/1.1",
  "headers": {
    "host": "localhost:8080",
    "user-agent": "curl/8.0",
    "accept": "*/*"
  },
  "timing": {
    "duration_ms": 0.042
  }
}
```

**Over HTTPS** the response additionally carries a `tls` object reporting what the connection negotiated — a fidelity win over httpbin/go-httpbin, and a way to confirm exactly what TLS an upstream negotiated behind a gateway:

```json
{
  "tls": {
    "version": "TLSv1.3",
    "cipher_suite": "TLS13_AES_256_GCM_SHA384",
    "alpn": "h2",
    "client_cert_present": false,
    "client_certs": []
  }
}
```

| `tls` field | Type | Description |
|-------------|------|-------------|
| `version` | string \| null | Negotiated protocol version, e.g. `"TLSv1.3"` |
| `cipher_suite` | string \| null | Negotiated cipher suite, e.g. `"TLS13_AES_256_GCM_SHA384"` |
| `alpn` | string \| null | Negotiated ALPN protocol (`"h2"` / `"http/1.1"`), or `null` if none |
| `client_cert_present` | bool | Whether the client presented a certificate (only under mTLS) |
| `client_certs` | array | Per-cert `{ "der_length": n }`, leaf-first; empty unless mTLS is configured |

### HEAD /get

Same as `GET /get` but returns headers only with an empty body.

**Response:** `200 OK` (empty body)

### POST /post

Echo request details including the parsed JSON body.

**Request body:** JSON (required)

**Response:** `200 OK`

| Field | Type | Description |
|-------|------|-------------|
| `method` | string | `"POST"` |
| `headers` | object | All request headers |
| `body` | any | The parsed JSON request body |
| `timing.duration_ms` | number | Processing time in milliseconds |

**Error:** `400 Bad Request` if the body is not valid JSON.

```json
{
  "method": "POST",
  "http_version": "HTTP/1.1",
  "headers": { "content-type": "application/json", "..." : "..." },
  "body": { "key": "value" },
  "timing": {
    "duration_ms": 0.058
  }
}
```

### PUT /put

Echo request details including the parsed JSON body.

**Request body:** JSON (required)

**Response:** `200 OK` — same shape as [POST /post](#post-post).

**Error:** `400 Bad Request` if the body is not valid JSON.

### PATCH /patch

Echo request details including the parsed JSON body.

**Request body:** JSON (required)

**Response:** `200 OK` — same shape as [POST /post](#post-post).

**Error:** `400 Bad Request` if the body is not valid JSON.

### DELETE /delete

Echo request details. Body is optional — if a JSON body is sent it is echoed; otherwise `body` is `null`.

**Request body:** JSON (optional)

**Response:** `200 OK`

| Field | Type | Description |
|-------|------|-------------|
| `method` | string | `"DELETE"` |
| `http_version` | string | HTTP version (e.g. `"HTTP/1.1"`, `"HTTP/2.0"`) |
| `headers` | object | All request headers |
| `body` | any \| null | Parsed JSON body, or `null` if none sent |
| `timing.duration_ms` | number | Processing time in milliseconds |

### OPTIONS /options

Returns allowed HTTP methods.

**Response:** `204 No Content`

| Header | Value |
|--------|-------|
| `Allow` | `GET, POST, PUT, PATCH, DELETE, OPTIONS, HEAD` |

---

## Utility Endpoints

Convenience endpoints that extract and return specific request properties.

### GET /uuid

Returns a randomly generated UUID v4.

**Response:** `200 OK`

| Field | Type | Description |
|-------|------|-------------|
| `uuid` | string | Random UUID v4 (e.g. `"550e8400-e29b-41d4-a716-446655440000"`) |
| `timing.duration_ms` | number | Processing time in milliseconds |

```json
{
  "uuid": "550e8400-e29b-41d4-a716-446655440000",
  "timing": {
    "duration_ms": 0.031
  }
}
```

### GET /ip

Returns the client's IP address. Checks `X-Forwarded-For` first (first entry if comma-separated), then `X-Real-IP`, defaulting to `"unknown"`.

**Response:** `200 OK`

| Field | Type | Description |
|-------|------|-------------|
| `origin` | string | Client IP address |
| `timing.duration_ms` | number | Processing time in milliseconds |

```json
{
  "origin": "192.168.1.100",
  "timing": {
    "duration_ms": 0.025
  }
}
```

### GET /user-agent

Returns the `User-Agent` header from the request.

**Response:** `200 OK`

| Field | Type | Description |
|-------|------|-------------|
| `user-agent` | string | User-Agent header value (empty string if not set) |
| `timing.duration_ms` | number | Processing time in milliseconds |

```json
{
  "user-agent": "curl/8.0",
  "timing": {
    "duration_ms": 0.022
  }
}
```

### GET /headers

Returns all request headers as a JSON object.

**Response:** `200 OK`

| Field | Type | Description |
|-------|------|-------------|
| `headers` | object | All request headers as key-value pairs |
| `timing.duration_ms` | number | Processing time in milliseconds |

```json
{
  "headers": {
    "host": "localhost:8080",
    "user-agent": "curl/8.0",
    "accept": "*/*",
    "authorization": "Bearer token123"
  },
  "timing": {
    "duration_ms": 0.028
  }
}
```

### GET /response-headers

Echoes each query parameter as a response header **and** in the JSON body. Useful for exercising gateway plugins that inspect or rewrite upstream response headers (Kong's `response-transformer`, `cors`, `proxy-cache`).

**Query parameters:** Any number of `key=value` pairs. Duplicate keys emit repeated `Set-Header`-style entries on the response and collapse to a JSON array in the body. User-supplied headers replace the default response headers (including `content-type`; the body is still JSON — intentional mismatch for plugin testing).

**Response:** `200 OK` — JSON body mirroring the headers.

```bash
curl -i 'http://localhost:8080/response-headers?x-rate-limit=100&cache-control=no-store'
```

```http
HTTP/1.1 200 OK
x-rate-limit: 100
cache-control: no-store

{
  "x-rate-limit": "100",
  "cache-control": "no-store"
}
```

**Errors:**

- `400 Bad Request` — invalid header name (must be valid HTTP token) or invalid header value (must be visible ASCII)

---

## Status Codes

### ANY /status/:code

Returns the specified HTTP status code. Accepts any HTTP method.

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `code` | u16 | HTTP status code to return (e.g. `200`, `404`, `503`) |

**Response:** The specified status code, with a JSON body `{ "status": <code>, "reason": "<canonical reason phrase>" }` (e.g. `{ "status": 404, "reason": "Not Found" }`). Unlike httpbin's empty body, the reason phrase is echoed for inspection.

**Error:** `400 Bad Request` if the code is not a valid HTTP status code.

```bash
curl -i http://localhost:8080/status/418
# HTTP/1.1 418 I'm a Teapot
```

---

## Wildcard

### ANY /anything

Echo any request regardless of HTTP method. Returns method, path, query string, headers, and body.

**Response:** `200 OK`

| Field | Type | Description |
|-------|------|-------------|
| `method` | string | The HTTP method used |
| `path` | string | Request path |
| `query` | string | Query string (empty string if none) |
| `headers` | object | All request headers |
| `body` | string | Raw request body as a string |
| `timing.duration_ms` | number | Processing time in milliseconds |
| `tls` | object | **HTTPS only** — negotiated TLS connection info (same shape as [GET /get](#get-get); omitted on plain HTTP) |

```json
{
  "method": "PUT",
  "http_version": "HTTP/1.1",
  "path": "/anything",
  "query": "foo=bar",
  "headers": { "..." : "..." },
  "body": "hello",
  "timing": {
    "duration_ms": 0.045
  }
}
```

### ANY /anything/*path

Same as `/anything` but captures an arbitrary subpath.

```bash
curl http://localhost:8080/anything/foo/bar/baz
# { "path": "/anything/foo/bar/baz", ... }
```

---

## Redirects

### ANY /redirect/:n

Returns a chain of HTTP 302 redirects that decrements on each hop.

- `/redirect/3` → 302 to `/redirect/2` → 302 to `/redirect/1` → 302 to `/get`
- `/redirect/0` → `200 OK` "Redirect complete"

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `n` | u32 | Number of redirect hops (max 20) |

**Responses:**

| Status | Condition | Description |
|--------|-----------|-------------|
| `302 Found` | `n >= 1` | `Location` → `/redirect/{n-1}` (or `/get` when `n=1`); `X-Redirect-Count` header carries the remaining hop count |
| `200 OK` | `n == 0` | Plain text "Redirect complete" |
| `400 Bad Request` | `n > 20` | Exceeds maximum allowed hops |

---

## Delay

### ANY /delay/:n

Delays the response by `n` seconds, then returns `200 OK`.

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `n` | u64 | Number of seconds to delay (max 300) |

**Responses:**

| Status | Condition | Description |
|--------|-----------|-------------|
| `200 OK` | `n <= 300` | Plain text "Response delayed by {n} seconds" |
| `400 Bad Request` | `n > 300` | Exceeds maximum allowed delay |

---

## Cookies

### GET /cookies

Returns all cookies from the request as a JSON object.

**Response:** `200 OK`

| Field | Type | Description |
|-------|------|-------------|
| `cookies` | object | Map of cookie name-value pairs |
| `timing.duration_ms` | number | Processing time in milliseconds |

```json
{
  "cookies": {
    "session": "abc123",
    "theme": "dark"
  },
  "timing": {
    "duration_ms": 0.033
  }
}
```

### GET /cookies/set

Sets cookies from query parameters and redirects to `/cookies`.

Each non-reserved query parameter becomes a `Set-Cookie` response header (default `Path=/`). Reserved keys add attributes applied to every cookie set in the request:

| Reserved key | Effect |
|--------------|--------|
| `secure` | adds `Secure` |
| `httponly` | adds `HttpOnly` |
| `samesite=<Strict\|Lax\|None>` | adds `SameSite=…` |
| `max_age=<seconds>` | adds `Max-Age=…` |
| `path=<path>` | overrides `Path` (default `/`) |
| `domain=<domain>` | adds `Domain=…` |

**Response:** `302 Found` with `Location: /cookies` and `Set-Cookie` headers.

```bash
curl -c - http://localhost:8080/cookies/set?name=rucho&lang=rust
# Set-Cookie: name=rucho; Path=/

# With attributes:
curl -i 'http://localhost:8080/cookies/set?session=abc&secure&httponly&samesite=Strict&max_age=3600'
# Set-Cookie: session=abc; Path=/; Max-Age=3600; SameSite=Strict; Secure; HttpOnly
```

### GET /cookies/delete

Deletes cookies by setting `Max-Age=0` and redirects to `/cookies`.

Each query parameter name identifies a cookie to expire. Values are ignored.

**Query parameters:** Cookie names to delete (values ignored).

**Response:** `302 Found` with `Location: /cookies` and expiring `Set-Cookie` headers.

```bash
curl -b "name=rucho" -c - http://localhost:8080/cookies/delete?name
# Set-Cookie: name=; Max-Age=0; Path=/
# Location: /cookies
```

---

## Data Formats

### GET /base64/:encoded

Decodes a URL-safe base64-encoded string from the URL path and returns metadata about the result.

**Path parameters:**

| Name | Description |
|------|-------------|
| `encoded` | URL-safe base64-encoded string (max 4096 bytes). Padding is optional. Standard base64 is attempted as a fallback but won't tolerate `/` in the path. |

**Response:** `200 OK`

| Field | Type | Description |
|-------|------|-------------|
| `encoded` | string | The input base64 string (as received) |
| `decoded` | string | The decoded content (via `String::from_utf8_lossy` — invalid UTF-8 bytes are replaced with `U+FFFD`) |
| `is_utf8` | boolean | `true` if the raw decoded bytes are valid UTF-8 |
| `byte_length` | number | Length of the decoded bytes |
| `timing.duration_ms` | number | Processing time in milliseconds |

**Errors:**

- `400 Bad Request` — invalid base64 input
- `400 Bad Request` — input exceeds the 4096-byte cap

```json
{
  "encoded": "SGVsbG8sIFJ1Y2hvIQ==",
  "decoded": "Hello, Rucho!",
  "is_utf8": true,
  "byte_length": 13,
  "timing": {
    "duration_ms": 0.041
  }
}
```

```bash
curl http://localhost:8080/base64/SGVsbG8sIFJ1Y2hvIQ==
```

### GET /bytes/:n

Returns `n` random bytes as `application/octet-stream`. The body is filled via `rand::thread_rng().fill_bytes()` for maximum entropy, which makes any tampering by an intermediate proxy observable.

**Path parameters:**

| Name | Description |
|------|-------------|
| `n` | Number of random bytes to return. Capped at 10 MiB (10 485 760). |

**Response:** `200 OK` — raw bytes with `Content-Type: application/octet-stream` and `Content-Length: n`. `n = 0` returns an empty 200.

**Errors:**

- `400 Bad Request` — `n` exceeds the 10 MiB cap

```bash
curl -o random.bin http://localhost:8080/bytes/1024
```

### GET /drip

Streams `numbytes` bytes of `*` evenly over `duration` seconds via `Transfer-Encoding: chunked`. Distinct from `/delay/:n`, which exercises *first-byte* (idle) timeouts: `/drip` exercises the *streaming* / inter-byte timeouts that fire when bytes are arriving but slowly (Kong's `read_timeout` / `send_timeout`, response buffering vs streaming behavior).

**Query parameters (all optional):**

| Name | Default | Description |
|------|---------|-------------|
| `numbytes` | `10` | Total bytes to emit. Capped at 10 000. |
| `duration` | `2` | Total stream duration in seconds. Capped at 300. |
| `code` | `200` | HTTP status code to return. Must be a valid HTTP status (100–999). |
| `delay` | `0` | Initial delay in seconds before the first byte. Capped at 300. |

**Response:** Status `code` (default `200`), `Content-Type: application/octet-stream`, body of `numbytes` `*` characters spread evenly across `duration` seconds. Chunk pacing is clamped so emissions are at least ~1 ms apart (tokio's timer precision).

**Errors:**

- `400 Bad Request` — any cap exceeded, or `code` is not a valid HTTP status

```bash
# 100 bytes spread over 5 seconds
curl http://localhost:8080/drip?numbytes=100&duration=5

# Test how a proxy handles a slow 504 upstream
curl -i 'http://localhost:8080/drip?numbytes=20&duration=3&code=504'
```

### GET /xml

Returns a small, valid sample XML document with `Content-Type: application/xml`. Deliberately non-JSON (like `/bytes`) — a controllable upstream for testing how a gateway handles XML responses (content-type-aware plugins, response transformers, compression of text bodies).

**Response:** `200 OK`, `Content-Type: application/xml`, a fixed sample XML body.

```bash
curl -i http://localhost:8080/xml
```

### GET /html

Returns a small, valid sample HTML document with `Content-Type: text/html; charset=utf-8`.

**Response:** `200 OK`, `Content-Type: text/html; charset=utf-8`, a fixed sample HTML body.

```bash
curl -i http://localhost:8080/html
```

### GET /image/:format

Returns a small, fixed 16×16 sample image in the requested `format` with the matching `Content-Type`. A controllable upstream for testing how a gateway handles binary/image responses — content-type routing and compression decisions (a gateway should generally skip re-compressing raster formats, but may compress the text-based SVG).

| `format` | Content-Type |
|----------|--------------|
| `png` | `image/png` |
| `jpeg` (alias `jpg`) | `image/jpeg` |
| `webp` | `image/webp` |
| `svg` | `image/svg+xml` |

**Errors:** `400 Bad Request` — any other format.

```bash
curl -o sample.png http://localhost:8080/image/png
curl -i http://localhost:8080/image/svg
```

### GET /range/:n

Returns `n` bytes of deterministic content (byte `i` is `a`+`i%26`, a repeating `a`..`z` pattern, so any slice is independently verifiable) with HTTP range-request support. A controllable upstream for testing how a gateway proxies partial-content / resumable downloads.

- **No `Range` header:** `200 OK`, full body, `Accept-Ranges: bytes`.
- **Satisfiable `Range`** (`bytes=start-end`, `bytes=start-`, `bytes=-suffix`): `206 Partial Content`, the requested slice, `Content-Range: bytes start-end/n`.
- **Unsatisfiable `Range`:** `416 Range Not Satisfiable`, `Content-Range: bytes */n`.

Only a single range is honored (the first if several are sent). `n` is capped at 10 MiB.

```bash
# Full body
curl -i http://localhost:8080/range/26

# First 5 bytes → 206, "abcde"
curl -i -H 'Range: bytes=0-4' http://localhost:8080/range/26

# Last 3 bytes → 206, "xyz"
curl -i -H 'Range: bytes=-3' http://localhost:8080/range/26
```

---

## Forced Content Encodings

### GET /gzip · /deflate · /brotli

Each returns a JSON echo of the request (`{ "<codec>": true, "method", "headers" }`) compressed with that codec and the matching `Content-Encoding`, **regardless of the request's `Accept-Encoding`** — the upstream *forces* the encoding. A controllable upstream for testing how a gateway handles an already-encoded body (e.g. Kong's Response-Transformer decode-and-rewrite path). The optional response-compression layer does **not** re-compress these (it skips bodies that already carry a `Content-Encoding`).

| Path | `Content-Encoding` | Body flag |
|------|--------------------|-----------|
| `/gzip` | `gzip` | `"gzipped": true` |
| `/deflate` | `deflate` (zlib) | `"deflated": true` |
| `/brotli` | `br` | `"brotli": true` |

```bash
# Fetch and decompress the gzip echo
curl -s http://localhost:8080/gzip | gunzip

# Confirm the Content-Encoding header (don't auto-decompress)
curl -s -D - -o /dev/null http://localhost:8080/brotli | grep -i content-encoding
```

---

## Conditional Caching

### GET /cache

Returns `304 Not Modified` if the request carries `If-None-Match` or `If-Modified-Since`; otherwise `200` with `ETag` + `Last-Modified` (and a JSON echo), so a client can revalidate on the next request. The validators are fixed/stable, so revalidation is deterministic.

### GET /cache/:n

Returns `200` with `Cache-Control: public, max-age=n` (seconds) and a JSON echo.

A controllable upstream for testing how a gateway proxies a *revalidating* upstream — Kong's `proxy-cache` plugin doesn't itself model 304/conditional revalidation, so it has to originate here.

```bash
# First request → 200 with ETag + Last-Modified
curl -i http://localhost:8080/cache
# Revalidate → 304 Not Modified
curl -i -H 'If-None-Match: "rucho-cache-v1"' http://localhost:8080/cache
# Freshness lifetime
curl -i http://localhost:8080/cache/60
```

---

## Infrastructure

### Response Headers

Set on every response by the middleware stack:

| Header | Description |
|--------|-------------|
| `X-Request-Id` | Correlation ID. Propagates a non-blank inbound `X-Request-Id`, otherwise mints a UUID v4. Toggle with `request_id_enabled` (default on). |
| `X-Response-Time` | Upstream processing time, e.g. `1.234ms` — the same value as the body's `timing.duration_ms`. |

### GET /healthz

Simple health check endpoint.

**Response:** `200 OK` — plain text `"OK"`.

### GET /metrics

Request statistics with all-time and rolling one-hour window. Only available when metrics are enabled (`RUCHO_METRICS_ENABLED=true`).

**Response:** `200 OK`

| Field | Type | Description |
|-------|------|-------------|
| `all_time.total_requests` | number | Total requests since server start |
| `all_time.successes` | number | Requests with 2xx/3xx status |
| `all_time.failures` | number | Requests with 4xx/5xx status |
| `all_time.endpoint_hits` | object | Per-endpoint hit counts |
| `last_hour.*` | | Same fields, rolling 60-minute window |

```json
{
  "all_time": {
    "total_requests": 1000,
    "successes": 950,
    "failures": 50,
    "endpoint_hits": {
      "/get": 500,
      "/post": 300
    }
  },
  "last_hour": {
    "total_requests": 100,
    "successes": 95,
    "failures": 5,
    "endpoint_hits": {
      "/get": 50,
      "/post": 30
    }
  }
}
```

### GET /endpoints

Returns a JSON array listing all available API endpoints.

**Response:** `200 OK`

Each entry contains:

| Field | Type | Description |
|-------|------|-------------|
| `path` | string | Endpoint path (e.g. `"/get"`) |
| `method` | string | HTTP method (e.g. `"GET"`, `"ANY"`) |
| `description` | string | Brief description of the endpoint |

### OPTIONS /options

See [OPTIONS /options](#options-options) under Echo Endpoints.

---

## Documentation

### GET /swagger-ui

Interactive OpenAPI/Swagger UI. Browse all endpoints, view schemas, and try requests from the browser.

### GET /api-docs/openapi.json

Raw OpenAPI 3.0 specification in JSON format. Use this to generate client SDKs or import into tools like Postman.
