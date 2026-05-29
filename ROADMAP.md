# Rucho — Project Roadmap

> **Rucho is an echo server first** — a faster, more robust httpbin replacement (Rust / Axum / Tokio) where speed, correctness, and high-fidelity request inspection are the core.
>
> **Secondarily, Rucho is a controllable testing upstream** to sit behind **Kong Gateway** or inside **Kong Mesh** (Kuma) — emitting stimuli that let you observe how the gateway/mesh proxies, transforms, times out, retries, caches, and routes.
>
> **Kong-redundancy principle:** build only upstream behaviors Kong *cannot* self-generate. If a Kong plugin or mesh policy already provides it (auth, rate limiting, gateway caching/compression, Prometheus, mTLS termination in mesh, request/response transformation), it is a **Non-Goal**.

Items are tagged **[H]** / **[M]** / **[L]** by priority.

---

## Completed

### Core echo & inspection
- [x] HTTP echo endpoints (GET, POST, PUT, PATCH, DELETE, OPTIONS, HEAD)
- [x] `/anything` wildcard (any method + subpaths)
- [x] `/status/:code` — return any HTTP status
- [x] `/delay/:n` — configurable first-byte delay (max 300 s)
- [x] `/healthz` — health check
- [x] `/endpoints` — self-documenting endpoint list
- [x] `/uuid`, `/ip` (peer-address fallback), `/user-agent`, `/headers`
- [x] Pretty-printed JSON output; graceful shutdown (SIGINT/SIGTERM); CLI (start/stop/status/version)

### Endpoints (echo-fidelity + upstream behaviors)
- [x] `/redirect/:n` — chained 302 redirects (max 20 hops)
- [x] `/cookies`, `/cookies/set`, `/cookies/delete`
- [x] `/base64/:encoded` — decode + JSON metadata (max 4 KiB)
- [x] `/response-headers` — echo query params as response headers
- [x] `/bytes/:n` — random bytes, `application/octet-stream` (max 10 MiB)
- [x] `/drip` — slow byte stream for inter-byte (read/send) timeout testing
- [x] `/xml`, `/html` — non-JSON content types (PR #132)
- [x] `/image/:format` — sample png/jpeg/svg/webp (PR #133)
- [x] `/range/:n` — `Accept-Ranges` / 206 partial content / 416 (PR #134)

### Protocol support
- [x] HTTP/1.1, HTTP/2 (with TLS), HTTPS via Rustls, TCP echo, UDP echo

### Performance & resilience
- [x] Response compression (gzip, brotli) — toggleable
- [x] Connection keep-alive + TCP socket tuning; zero-copy responses
- [x] Benchmark suite; zero-alloc metrics path normalization (`Cow<'static, str>`); thread-local chaos RNG (PR #131)
- [x] Chaos engineering mode (failure / delay / corruption injection)
- [x] Body-size cap on `/anything` (`max_body_size_bytes`, default 2 MiB) — closes OOM vector (PR #109)

### Infrastructure
- [x] Docker, Docker Compose, systemd, optimized multi-stage Dockerfile (189 MB), Docker Hub publishing
- [x] OpenAPI/Swagger UI; config files + env vars; PID file; GitHub Actions CI; permissive CORS
- [x] `/metrics` (JSON, toggleable); request tracing; request/response timing in echo responses

### Docs
- [x] Usage examples, man page (.deb), API reference, INTERNALS deep-dive (line-refs stripped, PR #108)

---

## T1 — Echo & Inspection Fidelity *(Primary mission)*

Make request inspection more correct, complete, and honest than httpbin & go-httpbin.

- [ ] **[H]** `/status/:code` returns the canonical reason phrase in the body (e.g. `"Not Found"` for 404) — httpbin-parity; an inspector should report the phrase, not just the number
- [ ] **[M]** Echo HTTP version + TLS info in `/get` / `/anything` (`http_version`, negotiated ALPN/cipher, presented client-cert info when available) — go-httpbin omits this; unique inspection value that doubles as gateway-proxy visibility
- [ ] **[M]** `/cache` + `/cache/:seconds` — emit `ETag` / `Last-Modified` and honor `If-None-Match` / `If-Modified-Since` → `304`; `/cache/:n` sets `Cache-Control: max-age=n`. Framed as *conditional-request fidelity* (the upstream emits the stimulus; lets you watch a gateway/cache react). Model on `range.rs`; no new deps
- [ ] **[M]** `parse_cookies` tolerates both `;` and `; ` separators (RFC 6265) — correctness; sloppy cookie parsing is a known httbin/go-httpbin pain point (`src/routes/cookies.rs`)
- [ ] **[M]** `/cookies/set` accepts attribute flags (`secure`, `httponly`, `samesite`, `max_age`) via query params — richer `Set-Cookie` fidelity for session inspection
- [ ] **[L]** Support `DELETE` on `/cookies` — API symmetry with `GET /cookies/delete`

---

## T2 — Gateway / Mesh Upstream Behaviors *(Secondary mission)*

Controllable upstream knobs to observe Kong Gateway / Kong Mesh behavior the gateway/mesh cannot self-generate.

- [ ] **[M]** Forced content-encoding trio `/gzip`, `/deflate`, `/brotli` — return a JSON body compressed with that codec + the matching `Content-Encoding`, *regardless of `Accept-Encoding`*. Tests Kong's Response-Transformer / RT-Advanced decode-and-rewrite path against an already-encoded upstream body (documented breakage: Kong/kong#13741, #1200). `flate2` + `brotli` are already transitive via tower-http → promote to direct deps (near-zero cost). One PR (trio). Fixed paths → no metrics-normalize change
- [ ] **[M]** `X-Response-Time` response header from `RequestTiming` — matches Kong's own plugin output; lets you compare upstream-measured vs gateway-measured latency
- [ ] **[M]** `/redirect/:n` emits an `X-Redirect-Count` header — observe hop number without parsing the URL; useful watching Kong follow-redirect behavior
- [ ] **[M]** Connection-control knob (e.g. `/anything?connection=close`) — force upstream `Connection: close` per request to observe Kong connection pooling / keep-alive reuse; the gateway cannot self-generate upstream teardown
- [ ] **[L]** mTLS — `ssl_ca_cert` config so the upstream *requires & verifies a client cert*. This is the way to test Kong's **upstream**-mTLS configuration (the gateway must present a cert the upstream demands). Distinct from mesh mTLS, which the sidecar terminates (see Non-Goals). Low priority — needs rustls client-cert verification
- [ ] **[L]** Investigate a slow-headers / slow-first-byte knob distinct from `/delay` and `/drip` — to exercise Kong upstream `read_timeout` vs `connect_timeout` separately. *Validate it isn't redundant with `/drip` (inter-byte) before building*

---

## T3 — Performance & Robustness

Keep the "fast, robust Rust" promise — hot-path correctness over premature optimization.

- [ ] **[M]** Metrics cardinality cap — bucket unknown `/cookies/{action}` (and any unmatched path) to a catch-all so a crawler/fuzzer can't grow the metrics map unbounded; mirrors existing `/delay`,`/bytes`,`/image`,`/range` normalization (`src/server/metrics_layer.rs`)
- [ ] **[M]** Metrics lock contention — swap `RwLock<HashMap>` for `DashMap` / sharded atomics. Only matters past ~10k rps; do it when a benchmark says so (`src/utils/metrics.rs`)
- [ ] **[L]** Replace `RwLock<usize>` around `current_bucket_idx` with `AtomicUsize` (`src/utils/metrics.rs`)
- [ ] **[L]** Replace `.unwrap()` in `head_handler` / `options_handler` response builders with `.expect("infallible: …")` — CLAUDE.md "no unwrap in production"
- [ ] **[L]** Remove the dead `500` branch in `endpoints_handler` — `serde_json::to_value` on a `&'static` slice is infallible
- [ ] **[L]** Handler boilerplate DRY — a non-macro `echo_with_body(method, headers, body)` helper for POST/PUT/PATCH/DELETE. Deferred is defensible; revisit only if touched
- [ ] **[L]** Module organization — move `src/tcp_udp_handlers.rs` → `src/server/echo.rs`; consider splitting `src/utils/`; move `ApiDoc` → `src/openapi.rs` (enables spec-shape integration tests)

---

## T4 — Testing & Quality

Coverage that backs the "more robust than httpbin" claim, and CI that catches the WSL-dev / Linux-CI drift.

- [x] **[H]** Add `windows-latest` to the CI matrix for `cargo check` — catches platform-gated drift; Rucho confirmed to compile cleanly on Windows (PR #136)
- [ ] **[H]** `spawn_full_app()` test helper that uses the real `build_app()` — current `spawn_app()` builds a minimal router and misses chaos/metrics middleware regressions
- [ ] **[M]** Integration-test gaps — `/delay` fires (≥1 s), `HEAD /get`, `/status/500`, response compression, `/metrics` enabled, `/endpoints` shape, malformed-JSON → 400
- [ ] **[M]** Property tests — chaos probabilities stay within bounds; `/redirect/:n` yields exactly `n` hops; `parse_cookies` never panics on any byte sequence
- [ ] **[M]** Benchmark gaps — `/anything` with a body, cookies roundtrip, metrics-contention concurrency, full middleware stack vs bare handler; benchmark `/redirect`
- [ ] **[L]** MSRV CI job pinning `rust-version` from `Cargo.toml` (resolve the CONTRIBUTING "1.70" vs Dockerfile "1.84" mismatch first — see T6)

---

## T5 — Build & Distribution

Docker/release ergonomics at **single-maintainer scope** — explicitly *not* production-team tooling (see `feedback_side_project_tooling_scope.md`).

- [ ] **[H]** Multi-arch Docker image (`linux/amd64,linux/arm64`) via `docker buildx` — small CI change, big UX for Apple-Silicon / ARM mesh nodes
- [ ] **[M]** `/healthz/ready` + `/healthz/live` — distinct K8s/mesh readiness vs liveness probes
- [ ] **[M]** Request-ID middleware — generate & return `X-Request-Id` on every response (pairs with gateway/mesh tracing as a correlation ID)
- [ ] **[M]** `log_format = json` config — `tracing_subscriber::fmt().json()` for structured-logging mesh deployments (Loki/Datadog/ELK)
- [ ] **[M]** Read-only-filesystem compatibility — PID path `/var/run/rucho` may break under `--read-only` Docker; make it tolerant/configurable (also the likely source of the stray `C:\var\run` artifact on Windows)
- [ ] **[M]** Auto-generated self-signed TLS certs (`ssl_auto_cert = true`, ephemeral in-memory via `rcgen`) — zero-setup HTTPS for dev/test; a test-ergonomics win, not gateway-redundant
- [ ] **[L]** Alpine image variant (smaller image)
- [ ] **[L]** Parallelize CI `deb` + `docker` jobs (both depend only on `build`)
- [ ] **[L]** Attach `SHA256SUMS` to GitHub releases — lightweight integrity, no recurring cost

---

## T6 — Documentation

Tell the dual-mission story and end the doc sprawl.

- [x] **[H]** "Why rucho?" section at the top of the README — vs httpbin / go-httpbin (speed, robustness, TCP/UDP, TLS, chaos) + the Kong-upstream pitch (PR #137)
- [x] **[H]** "Using rucho as a Kong upstream" section + a declarative `kong.yaml` snippet (PR #137)
- [x] **[M]** "Using rucho in Kong Mesh" snippet — Kuma sidecar injection + a MeshRetry example (PR #137)
- [ ] **[M]** Deduplicate the project-structure block (one canonical source; README/CONTRIBUTING/INTERNALS currently triplicate it — this ROADMAP no longer renders it either)
- [ ] **[M]** Deduplicate config-field tables — canonical source is `config_samples/rucho.conf.default`; link, don't re-render
- [ ] **[M]** Replace `docs/API_REFERENCE.md` with a one-pager linking `/swagger-ui` as canonical + 3–4 example responses (the hand-written table caused the v1.4.4 missing-endpoint fix)
- [ ] **[M]** Align MSRV — CONTRIBUTING says "Rust 1.70+", Dockerfile pins 1.84. Set `rust-version` in `Cargo.toml` and verify, or update the doc
- [ ] **[L]** Group README features under sub-headers (Protocol / Resilience / Observability / Deployment); explain why `compression_enabled` defaults off; mention the gitignored `tasks/` convention
- [ ] **[L]** Consolidate USAGE_EXAMPLES curl/Python/JS triplets (curl canonical; others in `<details>`)
- [ ] **[L]** Add `SECURITY.md` (disclosure + supported versions) and a lightweight `ARCHITECTURE.md`
- [ ] **[L]** CHANGELOG: add a "Performance" sub-category; add compare-link references at the bottom
- [ ] **[L]** `config_samples/rucho.conf.default` — consistent style (every field present, commented with its default)
- [ ] **[L]** Evaluate auto-generating internals from `///` doc comments (`cargo doc`); consider splitting INTERNALS by concern

---

## Priority Order

Ranked by payoff for the dual mission:

1. **`spawn_full_app()` real-`build_app()` test helper** — removes a correctness blind spot (chaos/metrics middleware) that undercuts the robustness claim
2. **Multi-arch Docker image** — small CI change, big UX for Apple-Silicon / ARM mesh nodes
3. **`/status/:code` reason phrase + `/redirect/:n` `X-Redirect-Count`** — cheap echo-fidelity + gateway-observability wins (two small one-PR-each items)
4. **Forced-encoding trio `/gzip`·`/brotli`·`/deflate`** — highest-value remaining endpoint; drives Kong Response-Transformer decode path; codecs already vendored
5. **Metrics cardinality cap + `/cache` conditional requests** — close the unbounded-metrics-key vector; add conditional-request (304) fidelity

_Done: `windows-latest` CI matrix (PR #136) · "Why rucho?" + Kong upstream/mesh docs (PR #137)._

---

## Backlog

Low-priority parked ideas — kept for reference, not scheduled.

- [ ] `/links/:n` — HTML page with `n` links. Primarily a client/crawler fixture with little gateway-upstream value; park unless an inspection-fidelity case emerges
- [ ] Extract echo-handler boilerplate via a macro — superseded by the non-macro helper idea in T3
- [ ] Non-JSON request-body echo for `/post` etc. (currently rejects non-JSON) — adds complexity for limited value

---

## Non-Goals

Out of scope to keep focus on the dual mission. Most are things **Kong Gateway or Kong Mesh already provides** (so a controllable upstream gains nothing by duplicating them), or identities Rucho isn't trying to be.

**Kong/mesh already does it:**
- Auth-validating endpoints (`/basic-auth`, `/bearer`, …) — Kong's `basic-auth`/`key-auth`/`jwt`/`oauth2` plugins validate credentials; `/headers` already exposes what the upstream received
- `/deny` and fixed-status endpoints — `/status/:code` already covers this with full flexibility
- Rate limiting — Kong's `rate-limiting` plugin owns this; for standalone use, document "put a gateway in front"
- Configurable CORS — a gateway response concern (Kong `cors` plugin); the permissive default is fine for a test upstream
- HSTS header — a gateway/edge security-posture concern (set via a gateway policy), not an upstream test stimulus
- Mesh mTLS termination — the Kong Mesh (Kuma) sidecar handles mTLS between services; duplicating it in the upstream adds nothing *(distinct from the optional upstream-mTLS test knob in T2, which targets Kong **Gateway** → upstream client-cert config)*
- Prometheus exposition for `/metrics` — Kong's Prometheus plugin + mesh observability cover gateway/mesh metrics; the JSON `/metrics` stays for quick introspection

**Supply-chain / production-team tooling (wrong scale for a single-maintainer test target):**
- Dependabot — tried (PR #117) and reverted (PR #129); within hours it opened 5 PRs, 3 needing triage for ecosystem incompatibilities. Quarterly manual `cargo update` is the right scale
- `cargo audit` / `cargo deny` CI jobs — the lockfile already carries known transitive advisories that would red-fail the enabling PR; run `cargo audit` manually, quarterly
- Container image vulnerability scanning (Trivy / Docker Scout) and SLSA build provenance — production supply-chain ceremony; revisit only if a downstream consumer requires it

**Not Rucho's identity:**
- gRPC support, plugin/extension systems, infrastructure provisioning (Terraform, etc.), request-replay features

---

## Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License — see [LICENSE](LICENSE).
