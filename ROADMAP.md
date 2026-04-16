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

### Endpoint Enhancements (from review)
- [ ] `/ip` peer-address fallback via `ConnectInfo<SocketAddr>` — currently returns `"unknown"` when no `X-Forwarded-For` / `X-Real-IP` header is present (`src/routes/core_routes.rs`)
- [ ] `/status/:code` should return the canonical reason phrase in the body (e.g., `"Not Found"` for 404) — matches httpbin behavior
- [ ] `/redirect/:n` should emit an `X-Redirect-Count` header so clients can observe hop number without parsing the URL
- [ ] `/cookies/set` should accept cookie attribute flags (`secure`, `httponly`, `samesite`, `max_age`) via query params for richer auth/session testing
- [ ] Support DELETE method for `/cookies` (API symmetry with GET `/cookies/delete`)

---

## Tier 4: Testing & Quality

- [x] Fix config test isolation (injectable env reader — v1.4.1)
- [x] Integration tests (spin up server, hit endpoints with reqwest, assert responses)
- [ ] Benchmark the redirect endpoint

### Integration Test Suite Gaps
- [ ] Test `/delay` (at least 1s to confirm the sleep fires)
- [ ] Test `HEAD /get` (header-only response path)
- [ ] Test `/status/500` and other error-range codes
- [ ] Test response compression (send `Accept-Encoding: gzip`, assert compressed body)
- [ ] Test `/metrics` endpoint when enabled
- [ ] Test `/endpoints` response shape
- [ ] Test POST/PUT/PATCH error path (malformed JSON → 400)
- [ ] Add `spawn_full_app()` helper that uses the real `build_app()` so middleware interactions are exercised — current `spawn_app()` builds a minimal router and would miss chaos/metrics regressions

### Benchmark Coverage
- [ ] Benchmark `/anything` with a body (exercises `to_bytes` path)
- [ ] Benchmark the cookies roundtrip (GET `/cookies` with a cookie header)
- [ ] Concurrency benchmark — expose the metrics `RwLock` contention ceiling under parallel load
- [ ] Benchmark the full middleware stack (chaos+metrics+timing+trace+compression) vs. bare handler

### Property Tests
- [ ] Add `proptest` for chaos middleware: for any valid `ChaosConfig`, rolled probabilities stay within declared bounds across N requests
- [ ] Add `proptest` for `/redirect/:n`: following the chain yields exactly `n` hops for any `n ∈ [0, MAX_REDIRECT_HOPS]`
- [ ] Add `proptest` for `parse_cookies`: never panics on any byte sequence in the `Cookie` header

### CI Matrix
- [ ] Add `windows-latest` to the CI matrix for `cargo check` — prevents WSL-dev vs Linux-CI drift on platform-gated code (e.g., `#[cfg(not(target_os = "windows"))]` in `src/server/http.rs`)
- [ ] Add MSRV CI job pinning `rust-version` from `Cargo.toml`

---

## Tier 5: Infrastructure & Ops

- [ ] `/healthz/ready` + `/healthz/live` — Kubernetes readiness vs liveness probes
- [ ] Request ID middleware — generate and return `X-Request-Id` on every response
- [ ] Configurable response size limit
- [ ] Alpine Docker image variant (smaller image)
- [ ] Auto-generated self-signed TLS certs (`ssl_auto_cert = true`) — ephemeral in-memory certs via `rcgen` for zero-setup HTTPS dev/testing
- [ ] Mutual TLS (mTLS) — `ssl_ca_cert` config for client certificate verification

### From Review
- [ ] Body-size cap on `/anything` handler — `to_bytes(body, usize::MAX)` is an OOM vector. Add `DefaultBodyLimit` layer (configurable via `max_body_size_bytes`) (`src/routes/core_routes.rs:305`)
- [ ] Prometheus exposition format for `/metrics` — emit `text/plain; version=0.0.4` alongside the JSON output, so users can scrape with Prometheus/Grafana (highest-ROI observability addition for gateway-upstream workflows)
- [ ] `log_format = json` config option — use `tracing_subscriber::fmt().json()` so the binary works in structured-logging environments (Loki, Datadog, ELK)
- [ ] `X-Response-Time` header — `RequestTiming` should emit this in addition to the extension, matching Kong's own plugin output
- [ ] Multi-arch Docker image (`linux/amd64,linux/arm64`) via `docker buildx` — benefits Apple Silicon users
- [ ] Parallelize CI `deb` and `docker` jobs — both depend on `build` but can run concurrently with each other
- [ ] Attach `SHA256SUMS` file to GitHub releases — supply-chain hygiene for binary/.deb downloads
- [ ] Container image vulnerability scanning in CI — Trivy or Docker Scout run against the published image
- [ ] Investigate read-only filesystem compatibility — PID file path `/var/run/rucho` may break under `--read-only` Docker runs

---

## Tier 6: Code Quality & Refactoring

### Correctness / Hot Paths
- [ ] Metrics lock contention — swap `RwLock<HashMap>` for `DashMap<String, AtomicU64>` or sharded atomics. Current implementation serializes every recorded request on a write lock (`src/utils/metrics.rs`)
- [ ] Metrics cardinality cap for `/cookies/{action}` — clients can force unbounded growth by sending arbitrary subpaths. Bucket unknown `action` values to `/cookies/other` (`src/server/metrics_layer.rs:52`)
- [ ] Chaos RNG: replace per-request `StdRng::from_rng(rand::thread_rng())` with cached `thread_local!` `StdRng` or use `rand::thread_rng()` directly — the v1.4.6 CHANGELOG claims this was optimized but the per-request seed is still present (`src/server/chaos_layer.rs:29`)
- [ ] Replace `RwLock<usize>` around `current_bucket_idx` with `AtomicUsize` (`src/utils/metrics.rs:78`)
- [ ] Remove dead 500 branch in `endpoints_handler` — `serde_json::to_value` on `&'static [EndpointInfo]` is infallible (`src/routes/core_routes.rs:454-465`)

### `.unwrap()` Hygiene
- [ ] Replace `.unwrap()` in `src/server/chaos_layer.rs:49-50, 55, 107, 118` with `.expect("infallible: static response builder")` so intent matches CLAUDE.md "no unwrap in production" rule
- [ ] Replace `.unwrap()` in `src/routes/core_routes.rs:431, 795` (head_handler, options_handler response builders) with `.expect(...)`

### DRY & Refactoring
- [ ] Consolidate POST/PUT/PATCH/DELETE handler boilerplate — the four handlers in `src/routes/core_routes.rs` differ only in method string; a `echo_with_body(method, headers, body)` helper would DRY them without a macro (reconsider the "deferred" decision in Backlog)
- [ ] `parse_cookies`: tolerate both `;` and `; ` separators — current `split("; ")` is stricter than RFC 6265 tolerance rules (`src/routes/cookies.rs:27`)

### Module Organization
- [ ] Move `src/tcp_udp_handlers.rs` out of `src/` root into `src/server/echo.rs` (or split into `tcp_echo.rs` / `udp_echo.rs`) — inconsistent with the rest of the layout
- [ ] Split `src/utils/` (10 unrelated modules) into themed submodules: `src/config/`, `src/http/` (response helpers), `src/process/` (pid), `src/observability/` (metrics, timing)
- [ ] Move `ApiDoc` out of `src/main.rs` into `src/openapi.rs` (library code) — binary-only code can't be tested, and integration tests could assert the spec shape
- [ ] Gate `anything_path_handler` behind `#[cfg(feature = "openapi-docs-only")]` or move to `openapi_stubs.rs` — it's documentation scaffolding, not runtime code (`src/routes/core_routes.rs:348-362`)

---

## Tier 7: Documentation

- [x] Usage examples doc — real-world testing scenarios (retries, redirects, timeouts)
- [x] Man page — ship with .deb package
- [x] API reference — auto-generated from OpenAPI spec
- [x] Internals deep-dive (`docs/INTERNALS.md`)

### INTERNALS.md Maintenance (2,357 lines — memory flags line-ref staleness as recurring pain)
- [ ] Strip specific `file:line` citations from INTERNALS.md — keep file paths only. Immediate maintenance-cost reduction; highest-ROI doc change
- [ ] Evaluate auto-generating internals from `///` doc comments (`cargo doc` + wrapper) — ~60% of INTERNALS duplicates existing doc comments
- [ ] Add a CI check validating any `file:line` citations in docs actually exist (file has ≥N lines)
- [ ] Split INTERNALS.md by concern: `ARCHITECTURE.md`, `MIDDLEWARE.md`, `CONFIG.md` — scoped updates, less drift

### README.md (378 lines)
- [ ] Add a "Why rucho?" differentiation section at the top — 3-4 lines comparing to httpbin, mockoon, go-httpbin (chaos mode, TCP/UDP, production-grade TLS + socket tuning)
- [ ] Deduplicate the config field tables — canonical source is `config_samples/rucho.conf.default`; README should link, not re-render
- [ ] Deduplicate the project-structure block (also lives in CONTRIBUTING.md and INTERNALS.md) — keep one canonical source
- [ ] Group the Features bullet list under sub-headers (Protocol, Resilience, Observability, Deployment) for faster scanning
- [ ] Explain why `compression_enabled` defaults to off ("preserves raw body inspection")
- [ ] Mention `tasks/` directory convention (gitignored, local tracking only)

### API_REFERENCE.md (468 lines)
- [ ] Replace with a one-pager that links to `/swagger-ui` as canonical and shows 3-4 example responses — the hand-written table is what caused the v1.4.4 "add 4 missing endpoints to OpenAPI" fix

### USAGE_EXAMPLES.md (923 lines)
- [ ] Consolidate the curl/Python/JavaScript triplets — pick curl as canonical and show the others only where behavior differs, or collapse non-curl examples in `<details>` blocks
- [ ] Add a "Using rucho as a Kong upstream" section with a declarative `kong.yaml` snippet — uniquely useful given the primary-user context

### CONTRIBUTING.md
- [ ] Deduplicate the "Project Structure" block (links to canonical source)
- [ ] Align MSRV — doc says "Rust 1.70+" but `Dockerfile` pins 1.84. Either set `rust-version = "1.70"` in `Cargo.toml` and verify, or update CONTRIBUTING to match actual MSRV
- [ ] Explain the "config tests use an injectable env reader" note — one sentence on the v1.4.1 isolation fix, so a future contributor doesn't reintroduce `--test-threads=1`

### CHANGELOG.md
- [ ] Add a "Performance" sub-category (distinct from neutral "Changed") — retroactively categorize v1.4.6 and v1.2.0 entries
- [ ] Add auto-diff link references at the bottom (`[1.4.6]: https://github.com/rumpus/rucho/compare/v1.4.5...v1.4.6`) — GitHub browsing convenience

### Config Sample (`config_samples/rucho.conf.default`)
- [ ] Pick a consistent style: every field present (commented-out with default as comment) vs. current mix of set values and `# Example:` hints. Uncomment-and-edit is easier than copy-from-README

### New Docs
- [ ] Add `SECURITY.md` — vulnerability disclosure process, supported versions
- [ ] Add `ARCHITECTURE.md` — lightweight ADR replacing most of INTERNALS.md for newcomer onboarding

### CLAUDE.md (local, gitignored)
- [ ] Add a "Debugging" section — `RUST_LOG=debug cargo run -- start`, where to find tracing output
- [ ] Note the `src/tcp_udp_handlers.rs` placement inconsistency so future readers aren't confused
- [ ] Add a "Gateway-upstream testing" section with Kong route config examples (project purpose context)

### ROADMAP.md (this file)
- [x] Renumber tiers to fill the Tier 6 gap (addressed by this revision)
- [ ] Add size/complexity tags `[S/M/L]` to items for contributor self-selection
- [ ] Resolve Status-column vs per-tier-header inconsistency in the Timeline table

---

## Tier 8: Security & Supply Chain

- [ ] Set `rust-version = "1.70"` in `Cargo.toml` `[package]` and verify — otherwise CONTRIBUTING's "Rust 1.70+" is aspirational
- [ ] Add `cargo audit` CI job (security advisories against `Cargo.lock`)
- [ ] Add `cargo deny` CI job (license + advisory policy enforcement)
- [ ] Add `.github/dependabot.yml` for Cargo + Docker + GitHub Actions — ~15 lines, saves weekly manual bumps
- [ ] Configurable CORS — gate the permissive default behind a `cors_allowed_origins` config field (comma-separated list, `*` preserved as opt-in)
- [ ] HSTS header for TLS listeners (`Strict-Transport-Security: max-age=...`)
- [ ] Rate limiting on standalone deploys — optional, or document that "a gateway should handle this" in README (since single client can saturate `/delay`)
- [ ] SLSA build provenance attestation on releases (optional; aligns with supply-chain best practice)

---

## Suggested Priority Order

Ranked by payoff-per-hour from the review:

1. **INTERNALS.md: strip line numbers** — immediate maintenance-cost reduction; memory flags this as recurring pain
2. **Body-size cap on `/anything`** — security, 5-line fix
3. **`/ip` peer-address fallback** — fixes a real correctness surprise, ~15 lines
4. **Prometheus metrics format** — unlocks Grafana dashboards, pairs naturally with Kong's Prom plugin
5. **`/basic-auth` + `/bearer` + `/response-headers`** — highest-ROI roadmap endpoints for Kong plugin testing
6. **`cargo audit` + Dependabot in CI** — supply-chain hygiene, trivial to add
7. **CI matrix adds `windows-latest`** — prevents the WSL-dev drift the memory flags
8. **Multi-arch Docker image** — small CI change, big UX win for Mac users
9. **Metrics lock contention (DashMap / sharded atomics)** — only matters past ~10k rps; do it when benchmarks say so
10. **Handler boilerplate DRY** — optional; the current "deferred" decision is defensible

---

## Backlog

Low-priority ideas — not worth the effort right now, but kept for reference.

- [ ] Extract echo handler boilerplate (macro or generic handler to DRY up POST/PUT/PATCH/DELETE patterns). Deferred: repetition is obvious not dangerous, and upcoming endpoints have unique logic that wouldn't fit a macro. (See Tier 6 for reconsideration under a non-macro helper-function approach.)
- [ ] Clean up `C:` directory at repo root — appears to be a WSL path-leak from a Windows tool; `.gitignore` pattern to prevent recurrence
- [ ] Request body echo for `/post` etc. could support non-JSON content types (currently rejects) — adds complexity for limited value

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
| Phase 7 | Testing, docs, code quality | 🔄 In progress |
| Phase 8 | Security & supply chain | 📋 Planned |

---

## Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## License

MIT License - see [LICENSE](LICENSE)
