# Rucho - Project Roadmap

> **Goal:** A highly robust, enterprise-grade, production-ready echo server built for extreme speed and performance.

---

## Completed

### Core Foundation
- [x] HTTP echo endpoints (GET, POST, PUT, PATCH, DELETE, OPTIONS, HEAD)
- [x] `/anything` wildcard endpoint (supports ANY method and subpaths)
- [x] `/status/:code` — return any HTTP status code
- [x] `/delay/:n` — configurable response delay (max 300s)
- [x] `/healthz` — health check endpoint
- [x] `/endpoints` — self-documenting endpoint list
- [x] Pretty-printed JSON output (default)
- [x] Graceful shutdown (SIGINT/SIGTERM)
- [x] CLI commands (start, stop, status, version)

### Protocol Support
- [x] HTTP/1.1
- [x] HTTP/2 (with TLS)
- [x] HTTPS via Rustls
- [x] TCP echo listener
- [x] UDP echo listener

### Utility Endpoints
- [x] `/uuid` — random UUID generation
- [x] `/ip` — client IP detection
- [x] `/user-agent` — User-Agent echo
- [x] `/headers` — request headers echo

### Production Infrastructure
- [x] Docker container builds
- [x] Docker Compose support
- [x] Systemd service integration
- [x] OpenAPI/Swagger documentation
- [x] Configuration via files and environment variables
- [x] PID file management
- [x] GitHub Actions CI pipeline
- [x] CORS support (permissive)
- [x] Docker Hub publishing (`rumpus/rucho`)
- [x] Optimized multi-stage Dockerfile (189MB image)

### Observability
- [x] `/metrics` endpoint (JSON format, toggleable)
- [x] Request tracing and logging
- [x] Request/response timing in echo responses (`timing.duration_ms`)

### Testing & Resilience
- [x] Chaos engineering mode (failure injection, delay injection, response corruption)

---

## Tier 1: Performance & Speed ✅

- [x] Response compression (gzip, brotli) — toggleable via config
- [x] Connection keep-alive tuning
- [x] Zero-copy response optimizations
- [x] Benchmark suite with performance baselines
- [x] Zero-alloc metrics path normalization (`Cow<'static, str>`)
- [x] Thread-local RNG seeding for chaos middleware

---

## Tier 2: Advanced Protocol Support ✅

- [x] `/redirect/:n` — chained redirects

---

## Tier 3: New Endpoints

### Cookies & Auth
- [x] `/cookies` + `/cookies/set` + `/cookies/delete` — inspect, set, and delete cookies
- [ ] `/basic-auth/:user/:pass` — test HTTP Basic auth (401 if wrong, 200 if correct)
- [ ] `/bearer` — test Bearer token auth (check `Authorization` header)

### Data Formats & Content Types
- [ ] `/base64/:encoded` — decode base64 in the URL and return the result
- [ ] `/bytes/:n` — return `n` random bytes (binary download testing)
- [ ] `/xml`, `/html` — return non-JSON content types
- [ ] `/image/:format` — return a small test image (png, jpeg, svg, webp)

### Response Control
- [ ] `/response-headers?key=value` — return arbitrary response headers via query params
- [ ] `/cache` + `/cache/:seconds` — return cache headers (`ETag`, `Last-Modified`, `Cache-Control`)
- [ ] `/gzip`, `/brotli`, `/deflate` — force a specific encoding regardless of `Accept-Encoding`
- [ ] `/deny` — return a 403 forbidden page

### Streaming & Range
- [ ] `/drip?duration=5&numbytes=10` — slowly drip data over time
- [ ] `/range/:n` — return `n` bytes with `Accept-Ranges` support
- [ ] `/links/:n` — return an HTML page with `n` links (crawler testing)

---

## Tier 4: Testing & Quality

- [x] Fix config test isolation (injectable env reader — v1.4.1)
- [x] Integration tests (spin up server, hit endpoints with reqwest, assert responses)
- [ ] Benchmark the redirect endpoint

---

## Tier 5: Infrastructure & Ops

- [ ] `/healthz/ready` + `/healthz/live` — Kubernetes readiness vs liveness probes
- [ ] Request ID middleware — generate and return `X-Request-Id` on every response
- [ ] Configurable response size limit
- [ ] Alpine Docker image variant (smaller image)
- [ ] Auto-generated self-signed TLS certs (`ssl_auto_cert = true`) — ephemeral in-memory certs via `rcgen` for zero-setup HTTPS dev/testing
- [ ] Mutual TLS (mTLS) — `ssl_ca_cert` config for client certificate verification

---

## Tier 7: Documentation ✅

- [x] Usage examples doc — real-world testing scenarios (retries, redirects, timeouts)
- [x] Man page — ship with .deb package
- [x] API reference — auto-generated from OpenAPI spec
- [x] Internals deep-dive (`docs/INTERNALS.md`)

---

## Backlog

Low-priority ideas — not worth the effort right now, but kept for reference.

- [ ] Extract echo handler boilerplate (macro or generic handler to DRY up POST/PUT/PATCH/DELETE patterns). Deferred: repetition is obvious not dangerous, and upcoming endpoints have unique logic that wouldn't fit a macro.

---

## Non-Goals

The following are explicitly out of scope to maintain focus on the core mission:

- Full authentication/authorization middleware (testing auth endpoints above are mock-only)
- gRPC support
- Plugin/extension systems
- Infrastructure provisioning (Terraform, etc.)
- Request replay features

---

## Timeline

| Phase | Focus | Status |
|-------|-------|--------|
| Phase 1 | Core echo functionality | ✅ Done |
| Phase 2 | Protocol support (HTTP/2, TLS, TCP/UDP) | ✅ Done |
| Phase 3 | Production infrastructure (Docker, systemd) | ✅ Done |
| Phase 4 | Performance optimizations | ✅ Done |
| Phase 5 | Advanced protocols (redirects) | ✅ Done |
| Phase 6 | New endpoints (cookies, auth, data formats) | 🔄 Next |
| Phase 7 | Testing & docs | 🔄 In progress |

---

## Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## License

MIT License - see [LICENSE](LICENSE)
