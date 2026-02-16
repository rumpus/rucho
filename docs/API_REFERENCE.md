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
- [Infrastructure](#infrastructure)
- [Documentation](#documentation)

---

## Echo Endpoints

Core echo handlers that reflect request details back to the caller. All echo responses include a `timing` object with processing duration.

### GET /get

Echo request details (method, headers, timing).

**Response:** `200 OK`

| Field | Type | Description |
|-------|------|-------------|
| `method` | string | `"GET"` |
| `headers` | object | All request headers as key-value pairs |
| `timing.duration_ms` | number | Processing time in milliseconds |

```json
{
  "method": "GET",
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

---

## Status Codes

### ANY /status/:code

Returns the specified HTTP status code. Accepts any HTTP method.

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `code` | u16 | HTTP status code to return (e.g. `200`, `404`, `503`) |

**Response:** The specified status code with an empty body.

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

```json
{
  "method": "PUT",
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
| `302 Found` | `n >= 1` | `Location` header points to `/redirect/{n-1}` or `/get` when `n=1` |
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

Each query parameter becomes a `Set-Cookie` response header with `Path=/`.

**Query parameters:** Arbitrary `name=value` pairs.

**Response:** `302 Found` with `Location: /cookies` and `Set-Cookie` headers.

```bash
curl -c - http://localhost:8080/cookies/set?name=rucho&lang=rust
# Set-Cookie: name=rucho; Path=/
# Set-Cookie: lang=rust; Path=/
# Location: /cookies
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

## Infrastructure

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
