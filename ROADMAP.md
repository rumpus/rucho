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
- [x] Pretty-printed JSON output; graceful shutdown on **SIGINT + SIGTERM** (drains in-flight requests, 5s grace); CLI (start/stop/status/version)

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
- [x] `/gzip`, `/deflate`, `/brotli` — forced `Content-Encoding` JSON echo (PR #142)
- [x] `/cache` + `/cache/:n` — conditional requests (304 / `ETag` / `Last-Modified` / `Cache-Control`) (PR #144)

### Protocol support
- [x] HTTP/1.1, HTTP/2 (with TLS), HTTPS via Rustls, TCP echo, UDP echo

### Performance & resilience
- [x] Response compression (gzip, brotli) — toggleable
- [x] Connection keep-alive + TCP socket tuning (HTTP listener; HTTPS/`bind_rustls` tuning pending — see T5); zero-copy responses
- [x] Slowloris protection — `header_read_timeout` config caps time to read request headers
- [x] Benchmark suite; zero-alloc metrics path normalization (`Cow<'static, str>`); thread-local chaos RNG (PR #131)
- [x] Chaos engineering mode (failure / delay / corruption injection)
- [x] Global request body-size cap — `DefaultBodyLimit` (`max_body_size_bytes`, default 2 MiB) on the whole router; closes the OOM vector (PR #109)

### Infrastructure
- [x] Docker, Docker Compose, systemd, optimized multi-stage Dockerfile (189 MB), Docker Hub publishing
- [x] OpenAPI/Swagger UI; config files + env vars; PID file; GitHub Actions CI; permissive CORS
- [x] `/metrics` (JSON, toggleable — not annotated in OpenAPI/`/endpoints` since it's toggle-gated; see T5); request tracing; request/response timing in echo responses

### Docs
- [x] Usage examples, man page (.deb), API reference, INTERNALS deep-dive (line-refs stripped, PR #108)

---

## T1 — Echo & Inspection Fidelity *(Primary mission)*

Make request inspection more correct, complete, and honest than httpbin & go-httpbin.

- [x] **[H]** `/status/:code` returns `{ "status", "reason" }` JSON carrying the canonical reason phrase (e.g. "Not Found" for 404) while the status line keeps the requested code — httpbin-parity inspection win (PR #140)
- [x] **[M]** Echo HTTP version (`http_version`) in all echo handlers (`/get`, `/anything`, `/post`, `/put`, `/patch`, `/delete`) — go-httpbin omits this; unique inspection value that doubles as gateway-proxy visibility (PR #151)
- [ ] **[M]** Echo TLS info in `/get` / `/anything` (negotiated version/ALPN/cipher, presented client-cert when available) — needs the HTTPS listener reworked: `axum_server::bind_rustls` doesn't surface the rustls connection to handlers, so this requires a custom tokio-rustls accept loop that injects TLS params into request extensions (bigger, HTTPS-path-only lift)
- [x] **[M]** `/cache` + `/cache/:n` — `/cache` returns `304` on `If-None-Match`/`If-Modified-Since`, else `200` + stable `ETag` + `Last-Modified`; `/cache/:n` sets `Cache-Control: public, max-age=n`. Conditional-request fidelity for watching a gateway/cache plugin react; no new deps (PR #144)
- [x] **[M]** `parse_cookies` tolerates both `;` and `; ` separators (RFC 6265) — now splits on `;` with whitespace trimming (PR #145)
- [x] **[M]** `/cookies/set` accepts attribute flags (`secure`, `httponly`, `samesite`, `max_age`, plus `path`/`domain`) via reserved query params — richer `Set-Cookie` fidelity for session inspection (PR #145)
- [ ] **[L]** Support `DELETE` on `/cookies` — API symmetry with `GET /cookies/delete`

---

## T2 — Gateway / Mesh Upstream Behaviors *(Secondary mission)*

Controllable upstream knobs to observe Kong Gateway / Kong Mesh behavior the gateway/mesh cannot self-generate.

- [x] **[M]** Forced content-encoding trio `/gzip`, `/deflate`, `/brotli` — JSON echo compressed with each codec + matching `Content-Encoding`, regardless of `Accept-Encoding` (tests Kong's Response-Transformer decode path). `flate2`+`brotli` promoted to direct deps; `CompressionLayer` verified not to double-encode (PR #142)
- [x] **[M]** `X-Response-Time` response header from `RequestTiming` (e.g. `1.234ms`, always on) — matches Kong's own plugin output; lets you compare upstream-measured vs gateway-measured latency (PR #152)
- [x] **[M]** `/redirect/:n` emits an `X-Redirect-Count` header (remaining hops) on each 302 — observe chain progress without parsing the URL (PR #140)
- [ ] **[M]** Connection-control knob (e.g. `/anything?connection=close`) — force upstream `Connection: close` per request to observe Kong connection pooling / keep-alive reuse; the gateway cannot self-generate upstream teardown
- [ ] **[L]** mTLS — `ssl_ca_cert` config so the upstream *requires & verifies a client cert*. This is the way to test Kong's **upstream**-mTLS configuration (the gateway must present a cert the upstream demands). Distinct from mesh mTLS, which the sidecar terminates (see Non-Goals). Low priority — needs rustls client-cert verification
- [ ] **[L]** Investigate a slow-headers / slow-first-byte knob distinct from `/delay` and `/drip` — to exercise Kong upstream `read_timeout` vs `connect_timeout` separately. *Validate it isn't redundant with `/drip` (inter-byte) before building*

---

## T3 — Performance & Robustness

Keep the "fast, robust Rust" promise — hot-path correctness over premature optimization.

- [x] **[M]** Metrics cardinality cap — `normalize_path` now buckets unknown `/cookies/{action}` → `/cookies/other` and any unmatched path → `/other` (via a `KNOWN_STATIC_PATHS` whitelist); also added the missing `/base64` arm and made every arm zero-allocation (PR #143)
- [ ] **[M]** Metrics lock contention — swap `RwLock<HashMap>` for `DashMap` / sharded atomics. Only matters past ~10k rps; do it when a benchmark says so (`src/utils/metrics.rs`)
- [ ] **[L]** Replace `RwLock<usize>` around `current_bucket_idx` with `AtomicUsize` (`src/utils/metrics.rs`)
- [ ] **[L]** Replace `.unwrap()` in `head_handler` / `options_handler` response builders with `.expect("infallible: …")` — CLAUDE.md "no unwrap in production"
- [ ] **[L]** Remove the dead `500` branch in `endpoints_handler` — `serde_json::to_value` on a `&'static` slice is infallible
- [ ] **[L]** Handler boilerplate DRY — a non-macro `echo_with_body(method, headers, body)` helper for POST/PUT/PATCH/DELETE. Deferred is defensible; revisit only if touched
- [ ] **[L]** Module organization — move `src/tcp_udp_handlers.rs` → `src/server/echo.rs`; consider splitting `src/utils/`; refresh INTERNALS' architecture-walkthrough prose still citing `build_app`/`ApiDoc` under `main.rs`. (`ApiDoc`→`src/openapi.rs` + `build_app`→`src/app.rs` landed in PR #138.)

---

## T4 — Testing & Quality

Coverage that backs the "more robust than httpbin" claim, and CI that catches the WSL-dev / Linux-CI drift.

- [x] **[H]** Add `windows-latest` to the CI matrix for `cargo check` — catches platform-gated drift; Rucho confirmed to compile cleanly on Windows (PR #136)
- [x] **[H]** `spawn_full_app()` test helper using the real `build_app()` — exposed `build_app`→`src/app.rs` and `ApiDoc`→`src/openapi.rs` in the library; 3 full-stack regression tests incl. one proving the metrics middleware records requests (PR #138)
- [ ] **[M]** Integration-test gaps — `/delay` fires (≥1 s), `HEAD /get`, response compression, `/endpoints` shape, malformed-JSON → 400. (`/metrics` enabled is already covered by the `spawn_full_app` tests; `/status/:code` only partially — just 418 is asserted.)
- [ ] **[M]** Property tests — chaos probabilities stay within bounds; `/redirect/:n` yields exactly `n` hops; `parse_cookies` never panics on any byte sequence
- [ ] **[M]** Benchmark gaps — `/anything` with a body, cookies roundtrip, metrics-contention concurrency, full middleware stack vs bare handler; benchmark `/redirect`
- [ ] **[L]** MSRV CI job pinning `rust-version` from `Cargo.toml` (resolve the CONTRIBUTING "1.70" vs Dockerfile "1.84" mismatch first — see T6)

---

## T5 — Build & Distribution

Docker/release ergonomics at **single-maintainer scope** — explicitly *not* production-team tooling (see `feedback_side_project_tooling_scope.md`).

- [x] **[H]** Multi-arch Docker image (`linux/amd64,linux/arm64`) via `docker buildx` + QEMU — `release.yml` pushes a multi-arch manifest at release time; PR CI does a fast amd64-only sanity build (arm64 validated at release, so PRs stay fast) (PRs #139, #141)
- [x] **[H]** SIGTERM graceful-shutdown handler — `shutdown.rs` now races `ctrl_c` with `tokio::signal::unix` `SignalKind::terminate()`, so Docker/K8s/Kong-Mesh SIGTERM drains in-flight requests (5s grace) instead of hard-killing. Unix-gated; non-Unix keeps SIGINT-only (PR #148)
- [x] **[M]** Request-ID middleware — sets `X-Request-Id` on every response (propagates a non-blank inbound id, else mints UUID v4; outermost layer; set-if-absent so handlers like `/response-headers` win). `request_id_enabled` toggle, default on (PR #147)
- [x] **[M]** `log_format = json` config — `tracing_subscriber::fmt().json()` for structured-logging mesh deployments (Loki/Datadog/ELK); `text` default, unknown value warns and falls back (PR #149)
- [x] **[M]** Read-only-filesystem compatibility — PID path is now configurable (`pid_file` / `RUCHO_PID_FILE`, default `/var/run/rucho/rucho.pid`) **and** a write failure is non-fatal: the server warns and starts anyway instead of aborting under `--read-only` Docker (PR #150)
- [ ] **[M]** Auto-generated self-signed TLS certs (`ssl_auto_cert = true`, ephemeral in-memory via `rcgen`) — zero-setup HTTPS for dev/test; a test-ergonomics win, not gateway-redundant
- [ ] **[L]** Alpine image variant (smaller image)
- [ ] **[L]** Parallelize CI `deb` + `docker` jobs (both depend only on `build`)
- [ ] **[L]** Attach `SHA256SUMS` to GitHub releases — lightweight integrity, no recurring cost
- [ ] **[L]** Apply TCP socket tuning to the HTTPS listener too — `configure_tcp_socket` currently runs only on the HTTP path, not the `bind_rustls` HTTPS listener (audit finding)
- [ ] **[L]** Annotate `/metrics` with `#[utoipa::path]` (and optionally list it in `/endpoints`) so it's discoverable in Swagger when enabled — currently invisible in both (audit finding)

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

1. **Echo TLS info in `/get`/`/anything`** — bigger lift: needs the HTTPS accept loop reworked to surface rustls connection params (see T1). The high-value, low-cost Priority Order items are now shipped; remaining work is this TLS rework plus the T4 (testing) / T6 (documentation) tiers

_Done: `windows-latest` CI (#136) · "Why rucho?" + Kong docs (#137) · `spawn_full_app()` + lib refactor (#138) · multi-arch Docker (#139) · `/status` + `/redirect` (#140) · amd64-only PR CI (#141) · forced-encoding trio (#142) · metrics cardinality cap (#143) · `/cache` (#144) · cookie fidelity (#145) · ROADMAP reconcile (#146) · request-id middleware (#147) · SIGTERM shutdown (#148) · `log_format=json` (#149) · read-only-FS PID compat (#150) · `http_version` echo (#151) · `X-Response-Time` header (#152)._

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
- `/healthz/ready` + `/healthz/live` split — for a stateless echo server, readiness and liveness are identical aliases of `/healthz`; the split adds surface without distinct semantics. Point both K8s/mesh probes at `/healthz`

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
