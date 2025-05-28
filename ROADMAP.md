# 🛣 Echo Server — Project Roadmap

---

## 🥇 Tier 1: Core Platform Improvements (Completed ✅)

- ✅ `/healthz` endpoint
- ✅ Optional pretty JSON output (`?pretty=true`)
- ✅ Graceful shutdown handling (SIGINT/SIGTERM)
- ✅ Support additional HTTP methods (HEAD, OPTIONS, ANY)

---

## 🥈 Tier 2: Developer Utility Endpoints (Completed ✅)

- ✅ `/delay/:n` — delay response by `n` seconds
- ✅ `/status/:code` — return specified HTTP status code (supports ANY method)
- [ ] WebSocket echo support
- [ ] gRPC echo server
- ✅ Add support for HTTP/2 (via Axum and Hyper, enabled with TLS)
- ✅ Add HTTPS support (via Rustls)

---

## 🥉 Tier 3: Productionization Features

- [ ] JSON structured server logs
- [ ] Panic recovery middleware
- [ ] Request/response size metrics
- ✅ Dockerfile for container builds
- [ ] Helm Chart for Kubernetes deployment
- ✅ OpenAPI/Swagger documentation

---

## 🚀 Future Bonus Ideas

- [ ] `/uuid` — return random UUID
- [ ] `/ip` — return requester IP
- [ ] `/user-agent` — return User-Agent
- [ ] `/headers` — echo headers
- [ ] `/redirect/:n` — perform chained redirects
- [ ] `/stream/:n` — stream multiple JSON objects
- [ ] Expose `/metrics` for Prometheus
- [ ] GitHub Actions (CI/CD automation)
- [ ] Request size limiting
- [ ] Implement connection pooling for better scalability
- [ ] Add rate limiting middleware to prevent abuse
- [ ] Implement CORS support for cross-origin requests
- [ ] Add authentication/authorization middleware (e.g., JWT or OAuth2)
- [ ] Add support for environment-based configuration (e.g., `.env` files)
- [ ] Provide Terraform scripts for cloud infrastructure provisioning
- [ ] Request replay feature
- [ ] Plugin system for extensibility (Lua, Wasm)

---

# 📢 Status

✅ Basic Echo Server working  
✅ `/anything` endpoint live (supports ANY method and subpaths)
✅ `/endpoints` endpoint lists all available API endpoints
✅ Modular routes and utils organized  
✅ Configuration loading via files and environment variables
✅ CORS support (permissive)
🚧 Tier 3 features under development  

---

# 📋 Contributing

Contributions, suggestions, and ideas are welcome!  
Feel free to fork and submit pull requests 🚀

---

# 📢 License

MIT License

---

# 📋 Timeline (Suggested)

| Phase | Focus |
|:---|:---|
| Phase 1 | ✅ Finish Tier 1 (Core improvements) |
| Phase 2 | ✅ Complete Tier 2 (Developer Utility Endpoints like `/delay/:n`, `/status/:code`, HTTPS, HTTP/2) |
| Phase 3 | ✅ Complete OpenAPI/Swagger documentation and Dockerfile (Tier 3) |
| Phase 4 | 🚧 Continue Tier 3 Productionization (Logs, Middleware, Helm) |
| Phase 5 | (Optional) Bonus Protocols like WebSockets, gRPC & Future Bonus Ideas |

---