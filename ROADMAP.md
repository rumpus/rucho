# Rucho - Project Roadmap

---

## Tier 1: Core Platform Improvements (Completed)

- [x] `/healthz` endpoint
- [x] Optional pretty JSON output (`?pretty=true`)
- [x] Graceful shutdown handling (SIGINT/SIGTERM)
- [x] Support additional HTTP methods (HEAD, OPTIONS, ANY)
- [x] CLI commands for server management (start, stop, status, version)

---

## Tier 2: Developer Utility Endpoints (Completed)

- [x] `/delay/:n` — delay response by `n` seconds (max 300s)
- [x] `/status/:code` — return specified HTTP status code (supports ANY method)
- [x] Add support for HTTP/2 (via Axum and Hyper, enabled with TLS)
- [x] Add HTTPS support (via Rustls)
- [x] TCP echo listener for protocol testing
- [x] UDP echo listener for protocol testing
- [ ] WebSocket echo support
- [ ] gRPC echo server

---

## Tier 3: Productionization Features

- [x] Dockerfile for container builds
- [x] Docker Compose support
- [x] Systemd service integration
- [x] OpenAPI/Swagger documentation
- [x] PID file management
- [x] Configuration via files and environment variables
- [ ] JSON structured server logs
- [ ] Panic recovery middleware
- [ ] Request/response size metrics
- [ ] Helm Chart for Kubernetes deployment

---

## Future Bonus Ideas

- [x] `/uuid` — return random UUID
- [x] `/ip` — return requester IP
- [x] `/user-agent` — return User-Agent
- [x] `/headers` — echo headers
- [ ] `/redirect/:n` — perform chained redirects
- [ ] `/stream/:n` — stream multiple JSON objects
- [x] `/metrics` — request statistics (basic JSON format)
- [ ] Expose `/metrics` for Prometheus (extended format)
- [x] GitHub Actions (CI/CD automation)
- [ ] Add rate limiting middleware to prevent abuse
- [ ] Add authentication/authorization middleware (e.g., JWT or OAuth2)
- [ ] Provide Terraform scripts for cloud infrastructure provisioning
- [ ] Request replay feature
- [ ] Plugin system for extensibility (Lua, Wasm)

---

## Status

**Completed:**
- Basic Echo Server with all HTTP methods
- `/anything` endpoint (supports ANY method and subpaths)
- `/endpoints` self-documenting endpoint list
- Modular codebase (cli/, server/, routes/, utils/)
- Configuration via files and environment variables
- CORS support (permissive)
- TCP/UDP echo listeners
- Docker and Docker Compose
- Systemd integration
- OpenAPI/Swagger UI
- GitHub Actions CI pipeline
- `/metrics` endpoint (basic JSON format, toggleable)
- Utility endpoints: `/uuid`, `/ip`, `/user-agent`, `/headers`

**In Progress:**
- Tier 3 productionization (structured logs, Helm)

---

## Timeline

| Phase   | Focus                                                    | Status |
|---------|----------------------------------------------------------|--------|
| Phase 1 | Core improvements (healthz, methods, shutdown)           | Done   |
| Phase 2 | Developer endpoints (delay, status, HTTPS, TCP/UDP)      | Done   |
| Phase 3 | Productionization (Docker, Swagger, systemd)             | Done   |
| Phase 4 | Advanced productionization (logs, metrics, Helm)         | Next   |
| Phase 5 | Bonus features (WebSocket, gRPC, Prometheus)             | Future |

---

## Contributing

Contributions, suggestions, and ideas are welcome!
See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## License

MIT License - see [LICENSE](LICENSE)
