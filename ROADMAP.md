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

## Tier 1: Performance & Speed (Next)

- [x] Response compression (gzip, brotli) — toggleable via config
- [ ] Connection keep-alive tuning
- [ ] Zero-copy response optimizations
- [ ] Benchmark suite with performance baselines

---

## Tier 2: Enterprise Observability

- [ ] JSON structured server logs
- [ ] Prometheus metrics format (`/metrics`)
- [ ] Request/response size metrics
- [ ] OpenTelemetry tracing support
- [ ] Latency percentile tracking (p50, p95, p99)

---

## Tier 3: Production Deployment

- [ ] Helm Chart for Kubernetes
- [ ] Horizontal scaling documentation
- [ ] Resource limit recommendations
- [ ] High-availability configuration guide

---

## Tier 4: Advanced Protocol Support

- [ ] WebSocket echo support
- [ ] HTTP/3 (QUIC) support
- [ ] `/redirect/:n` — chained redirects
- [ ] `/stream/:n` — streaming JSON responses

---

## Tier 5: Security & Resilience

- [ ] Rate limiting middleware
- [ ] Request size limits (configurable)
- [ ] Connection limits per IP
- [ ] Slow client timeout handling

---

## Non-Goals

The following are explicitly out of scope to maintain focus on the core mission:

- Authentication/authorization middleware
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
| Phase 4 | Performance optimizations | 🔄 Next |
| Phase 5 | Enterprise observability | Planned |
| Phase 6 | Advanced protocols (WebSocket, HTTP/3) | Future |

---

## Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## License

MIT License - see [LICENSE](LICENSE)
