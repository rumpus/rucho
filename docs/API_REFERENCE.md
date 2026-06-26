# API Reference

Rucho's **canonical, always-accurate API specification** is served from the
running server and generated directly from the code (via `utoipa`), so it never
drifts from the implementation:

- **`/swagger-ui`** — interactive Swagger UI: browse every endpoint, view
  schemas, and try requests from the browser.
- **`/api-docs/openapi.json`** — the raw OpenAPI 3.0 spec, for generating client
  SDKs or importing into Postman / Insomnia.

```bash
cargo run -- start
# then open http://localhost:8080/swagger-ui
#  or fetch http://localhost:8080/api-docs/openapi.json
```

For the at-a-glance list of every route, see the
**[API Endpoints table in the README](../README.md#api-endpoints)**. For
copy-paste usage in curl / Python / JavaScript, see
**[USAGE_EXAMPLES.md](USAGE_EXAMPLES.md)**.

The rest of this page is a quick orientation — a few representative responses so
you can see the shape of rucho's JSON without running the server. The
`/swagger-ui` spec above is authoritative for the full set.

## Example responses

### `GET /get` — echo request details

```json
{
  "method": "GET",
  "http_version": "HTTP/1.1",
  "headers": {
    "host": "localhost:8080",
    "user-agent": "curl/8.0",
    "accept": "*/*"
  },
  "timing": { "duration_ms": 0.042 }
}
```

Over **HTTPS**, `/get` and `/anything` additionally include a `tls` object
describing the negotiated connection (omitted on plain HTTP):

```json
"tls": {
  "version": "TLSv1.3",
  "cipher_suite": "TLS13_AES_256_GCM_SHA384",
  "alpn": "h2",
  "client_cert_present": false,
  "client_certs": []
}
```

### `ANY /anything` — echo any request (method, path, query, headers, body)

```json
{
  "method": "POST",
  "http_version": "HTTP/1.1",
  "path": "/anything",
  "query": "foo=bar",
  "headers": { "...": "..." },
  "body": "hello",
  "timing": { "duration_ms": 0.045 }
}
```

Add `?connection=close` to force a `Connection: close` response (HTTP/1.1 only;
ignored over HTTP/2): the server hangs up after replying and echoes
`"connection": "close"` in the body — for observing how a gateway handles
upstream connection teardown and keep-alive reuse.

### `ANY /status/:code` — return a chosen status code

Returns the requested status line with a JSON body carrying the canonical reason
phrase — an inspection win over httpbin's empty body. `400` if the code isn't a
valid HTTP status.

```bash
curl -i http://localhost:8080/status/404
# HTTP/1.1 404 Not Found
# { "status": 404, "reason": "Not Found" }
```

## Response headers

Set on every response by the middleware stack:

| Header | Description |
|--------|-------------|
| `X-Request-Id` | Correlation ID. Propagates a non-blank inbound `X-Request-Id`, otherwise mints a UUID v4. Toggle with `request_id_enabled` (default on). |
| `X-Response-Time` | Upstream processing time, e.g. `1.234ms` — the same value as the body's `timing.duration_ms`. |
