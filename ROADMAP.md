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
- [x] Docker, Docker Compose, systemd, optimized multi-stage Dockerfile (~189 MB), Docker Hub publishing
- [x] OpenAPI/Swagger UI; config files + env vars; PID file; GitHub Actions CI; permissive CORS
- [x] `/metrics` (JSON, toggleable — not annotated in OpenAPI/`/endpoints` since it's toggle-gated; see T5); request tracing; request/response timing in echo responses

### Docs
- [x] Usage examples, man page (.deb), API reference, INTERNALS deep-dive (line-refs stripped, PR #108)

---

## T1 — Echo & Inspection Fidelity *(Primary mission)*

Make request inspection more correct, complete, and honest than httpbin & go-httpbin.

- [x] **[H]** `/status/:code` returns `{ "status", "reason" }` JSON carrying the canonical reason phrase (e.g. "Not Found" for 404) while the status line keeps the requested code — httpbin-parity inspection win (PR #140)
- [x] **[M]** Echo HTTP version (`http_version`) in all echo handlers (`/get`, `/anything`, `/post`, `/put`, `/patch`, `/delete`) — go-httpbin omits this; unique inspection value that doubles as gateway-proxy visibility (PR #151)
- [x] **[M]** Echo TLS info in `/get` / `/anything` (negotiated version/ALPN/cipher, presented client-cert when available) — a `TlsInfoAcceptor` wraps axum-server's `RustlsAcceptor` to read the handshaken `rustls::ServerConnection` (which `bind_rustls` hides) and inject a `TlsConnectionInfo` request extension; handlers echo it under `tls` over HTTPS, omit it on plain HTTP. HTTP/2 ALPN + graceful shutdown preserved (PR #155)
- [x] **[M]** `/cache` + `/cache/:n` — `/cache` returns `304` on `If-None-Match`/`If-Modified-Since`, else `200` + stable `ETag` + `Last-Modified`; `/cache/:n` sets `Cache-Control: public, max-age=n`. Conditional-request fidelity for watching a gateway/cache plugin react; no new deps (PR #144)
- [x] **[M]** `parse_cookies` tolerates both `;` and `; ` separators (RFC 6265) — now splits on `;` with whitespace trimming (PR #145)
- [x] **[M]** `/cookies/set` accepts attribute flags (`secure`, `httponly`, `samesite`, `max_age`, plus `path`/`domain`) via reserved query params — richer `Set-Cookie` fidelity for session inspection (PR #145)
- [x] **[L]** Support `DELETE` on `/cookies` — `DELETE /cookies?name…` expires the named cookies (`Max-Age=0`) and 302-redirects to `/cookies`, mirroring `GET /cookies/delete` via a shared `expire_cookies` helper (PR #174)

---

## T2 — Gateway / Mesh Upstream Behaviors *(Secondary mission)*

Controllable upstream knobs to observe Kong Gateway / Kong Mesh behavior the gateway/mesh cannot self-generate.

- [x] **[M]** Forced content-encoding trio `/gzip`, `/deflate`, `/brotli` — JSON echo compressed with each codec + matching `Content-Encoding`, regardless of `Accept-Encoding` (tests Kong's Response-Transformer decode path). `flate2`+`brotli` promoted to direct deps; `CompressionLayer` verified not to double-encode (PR #142)
- [x] **[M]** `X-Response-Time` response header from `RequestTiming` (e.g. `1.234ms`, always on) — matches Kong's own plugin output; lets you compare upstream-measured vs gateway-measured latency (PR #152)
- [x] **[M]** `/redirect/:n` emits an `X-Redirect-Count` header (remaining hops) on each 302 — observe chain progress without parsing the URL (PR #140)
- [x] **[M]** Connection-control knob — `/anything?connection=close` sets a `Connection: close` response header (HTTP/1.1 only; reflected-but-not-honored over HTTP/2, where `Connection` is a forbidden header), so hyper closes the socket after the response and a gateway in front is observed re-establishing rather than reusing the upstream connection. Per-request query directive (no config toggle), parsed from the raw query so `/anything` stays permissive; the honored outcome is echoed under a `connection` key. The gateway cannot self-generate upstream teardown (PR #170)
- [ ] **[L]** mTLS — `ssl_ca_cert` config so the upstream *requires & verifies a client cert*. This is the way to test Kong's **upstream**-mTLS configuration (the gateway must present a cert the upstream demands). Distinct from mesh mTLS, which the sidecar terminates (see Non-Goals). Low priority — needs rustls client-cert verification
- [ ] **[L]** Investigate a slow-headers / slow-first-byte knob distinct from `/delay` and `/drip` — to exercise Kong upstream `read_timeout` vs `connect_timeout` separately. *Validate it isn't redundant with `/drip` (inter-byte) before building*

---

## T3 — Performance & Robustness

Keep the "fast, robust Rust" promise — hot-path correctness over premature optimization.

- [x] **[M]** Metrics cardinality cap — `normalize_path` now buckets unknown `/cookies/{action}` → `/cookies/other` and any unmatched path → `/other` (via a `KNOWN_STATIC_PATHS` whitelist); also added the missing `/base64` arm and made every arm zero-allocation (PR #143)
- [ ] **[M]** Metrics lock contention — swap `RwLock<HashMap>` for `DashMap` / sharded atomics. Only matters past ~10k rps; do it when a benchmark says so (`src/utils/metrics.rs`)
- [x] **[L]** Replaced `RwLock<usize>` around `current_bucket_idx` with `AtomicUsize` — it was only ever touched inside the `rolling_buckets` write lock, so the lock already serializes it; Relaxed atomics drop a lock acquisition per request (PR #167)
- [x] **[L]** Replaced `.unwrap()` in `head_handler` / `options_handler` response builders with `.expect("infallible: …")` — CLAUDE.md "no unwrap in production" (PR #167)
- [x] **[L]** Removed the dead `500` branch in `endpoints_handler` (and its now-impossible `500` from the OpenAPI annotation) — `serde_json::to_value` on a `&'static` slice is infallible (PR #167)
- [ ] **[L]** Handler boilerplate DRY — a non-macro `echo_with_body(method, headers, body)` helper for POST/PUT/PATCH/DELETE. Deferred is defensible; revisit only if touched
- [ ] **[L]** Module organization — move `src/tcp_udp_handlers.rs` → `src/server/echo.rs`; consider splitting `src/utils/`. (INTERNALS' architecture prose that still cited `build_app`/`ApiDoc` under `main.rs` was corrected in the v1.5.0 docs sweep; the code move itself — `ApiDoc`→`src/openapi.rs`, `build_app`→`src/app.rs` — landed in PR #138.)

---

## T4 — Testing & Quality

Coverage that backs the "more robust than httpbin" claim, and CI that catches the WSL-dev / Linux-CI drift.

- [x] **[H]** Add `windows-latest` to the CI matrix for `cargo check` — catches platform-gated drift; Rucho confirmed to compile cleanly on Windows (PR #136)
- [x] **[H]** `spawn_full_app()` test helper using the real `build_app()` — exposed `build_app`→`src/app.rs` and `ApiDoc`→`src/openapi.rs` in the library; 3 full-stack regression tests incl. one proving the metrics middleware records requests (PR #138)
- [x] **[M]** Integration-test gaps — added `/delay` fires (≥1 s) + over-cap 400, `HEAD /get` empty body, response compression (gzip via a compression-enabled app), `/endpoints` shape, malformed-JSON → 400, and fuller `/status/:code` reason-phrase coverage (404/500/200, not just 418) (PR #156)
- [x] **[M]** Property tests (`proptest`) — chaos roll stays in `[0,1)` and a `0.0` rate never fires; `/redirect/:n` points exactly one hop closer for all in-range `n` (so the chain is exactly `n` hops); `parse_cookies` never panics and never yields an empty cookie name (PR #157)
- [x] **[M]** Benchmark gaps — added `POST /anything` (with body), cookies set+read roundtrip, `GET /redirect/3`, `GET /get` through the full middleware stack (vs the bare handler, to quantify overhead), and a 4-task concurrent `Metrics::record_request` contention bench (the baseline a DashMap/sharded swap would beat) (PR #158)
- [x] **[L]** MSRV CI job — a dedicated `MSRV (1.84)` job (`dtolnay/rust-toolchain@1.84.0` + `cargo check --all-features`, lib+bins only so dev-dep MSRVs don't leak in) now fails an MSRV break at PR time instead of only at the `rust:1.84` release Docker build. Verified the shipped crate compiles on 1.84.0 locally before adding (PR #159)

---

## T5 — Build & Distribution

Docker/release ergonomics at **single-maintainer scope** — explicitly *not* production-team tooling (see the **Non-Goals → Supply-chain / production-team tooling** section below).

- [x] **[H]** Multi-arch Docker image (`linux/amd64,linux/arm64`) via `docker buildx` + QEMU — `release.yml` pushes a multi-arch manifest at release time; PR CI does a fast amd64-only sanity build (arm64 validated at release, so PRs stay fast) (PRs #139, #141)
- [x] **[H]** SIGTERM graceful-shutdown handler — `shutdown.rs` now races `ctrl_c` with `tokio::signal::unix` `SignalKind::terminate()`, so Docker/K8s/Kong-Mesh SIGTERM drains in-flight requests (5s grace) instead of hard-killing. Unix-gated; non-Unix keeps SIGINT-only (PR #148)
- [x] **[M]** Request-ID middleware — sets `X-Request-Id` on every response (propagates a non-blank inbound id, else mints UUID v4; outermost layer; set-if-absent so handlers like `/response-headers` win). `request_id_enabled` toggle, default on (PR #147)
- [x] **[M]** `log_format = json` config — `tracing_subscriber::fmt().json()` for structured-logging mesh deployments (Loki/Datadog/ELK); `text` default, unknown value warns and falls back (PR #149)
- [x] **[M]** Read-only-filesystem compatibility — PID path is now configurable (`pid_file` / `RUCHO_PID_FILE`, default `/var/run/rucho/rucho.pid`) **and** a write failure is non-fatal: the server warns and starts anyway instead of aborting under `--read-only` Docker (PR #150)
- [x] **[M]** Auto-generated self-signed TLS certs (`ssl_auto_cert = true`, ephemeral in-memory via `rcgen`) — when set, the HTTPS listener serves a generated self-signed cert (`localhost`/`127.0.0.1`/`::1`) instead of requiring `ssl_cert`/`ssl_key` files; explicit files take precedence. Zero-setup HTTPS for dev/test (PR #178)
- [ ] **[L]** Alpine image variant (smaller image)
- [x] **[L]** Parallelize CI `deb` + `docker` jobs — already satisfied: both are `needs: build` in `ci.yml`, so they run concurrently once `build` passes (verified during the v1.5.0 quick-wins pass)
- [x] **[L]** Attach `SHA256SUMS` to GitHub releases — `release.yml` writes a `SHA256SUMS` (release binary + `.deb`, listed by basename) and attaches it; verify with `sha256sum -c SHA256SUMS` (PR #177)
- [x] **[L]** Apply TCP socket tuning to the HTTPS listener — the HTTPS path now binds + `configure_tcp_socket`s the listener (keep-alive, `TCP_NODELAY`) before attaching the TLS-info acceptor via `from_tcp`, matching the HTTP path (it had used `Server::bind`, which binds internally and skipped the tuning) (PR #176)
- [x] **[L]** Annotated `/metrics` with `#[utoipa::path]` + registered it in `ApiDoc`, so it's now discoverable in Swagger / `openapi.json` (the response description notes it's only mounted when `metrics_enabled`). Deliberately left out of `/endpoints`, which lists always-mounted routes (PR #175)

---

## T6 — Documentation

Tell the dual-mission story and end the doc sprawl.

- [x] **[H]** "Why rucho?" section at the top of the README — vs httpbin / go-httpbin (speed, robustness, TCP/UDP, TLS, chaos) + the Kong-upstream pitch (PR #137)
- [x] **[H]** "Using rucho as a Kong upstream" section + a declarative `kong.yaml` snippet (PR #137)
- [x] **[M]** "Using rucho in Kong Mesh" snippet — Kuma sidecar injection + a MeshRetry example (PR #137)
- [x] **[M]** Deduplicate the project-structure block — README's tree is now the single canonical source; CONTRIBUTING points to it instead of carrying an identical copy. INTERNALS keeps its deeper architecture-oriented tree (a distinct artifact, not a verbatim dup) (PR #160)
- [x] **[M]** Deduplicate config-field tables — removed the drift-prone third copy: INTERNALS §7.2 "Complete Field Reference" now points to the README's Parameters + Chaos tables (the canonical doc reference), with types in the §7.1 struct and `config_samples/rucho.conf.default` as the runnable example. (Kept the README table rather than gutting it to a bare link, since it carries the user-facing descriptions — maintainer call) (PR #161)
- [x] **[M]** Replace `docs/API_REFERENCE.md` with a one-pager — 739-line hand-written reference (the source of the v1.4.4 drift bug) slimmed to ~95 lines: `/swagger-ui` + `/api-docs/openapi.json` as the canonical generated spec, a link to the README endpoint table, three example responses (`/get` incl. `tls`, `/anything`, `/status/:code`), and the response-headers table (PR #162)
- [x] **[M]** Align MSRV — set `rust-version = "1.84"` in `Cargo.toml` (the `rust:1.84` release Docker image builds the project, verifying it compiles) and updated CONTRIBUTING from the stale "1.70+" to "1.84+" (PR #153)
- [x] **[L]** Grouped README features under sub-headers (Echo & inspection / Controllable upstream behaviors / Protocol & connection / Observability / Deployment & ops); filled in previously-unlisted features (`/metrics`, the inspection endpoints); explained why `compression_enabled` defaults off; noted the gitignored `tasks/` convention in CONTRIBUTING (PR #163)
- [~] **[L]** ~~Consolidate USAGE_EXAMPLES curl/Python/JS triplets~~ — **won't do**: ~17 `<details>` wraps across a 1411-line file is high churn for a purely cosmetic gain, and the triplets are useful as-is (maintainer call to skip)
- [x] **[L]** Added a minimal `SECURITY.md` — test-target scope framing, latest-release-only support, and private GitHub vulnerability reporting (no over-promised SLA). `ARCHITECTURE.md` deliberately skipped: `docs/INTERNALS.md` already is the architecture doc, and a second one would re-introduce the sprawl T6 just removed (maintainer call) (PR #166)
- [x] **[L]** CHANGELOG: added a `### Performance` sub-category (moved the chaos-RNG optimization there from Changed) and compare-link references at the bottom for every tagged release (`v1.0.0`…`v1.4.6` + `Unreleased`); `0.1.0`/`0.0.1` were never tagged so they stay plain (PR #165)
- [x] **[L]** `config_samples/rucho.conf.default` — consistent style: every field is now shown commented-out at its default with a one-line description (was a mix of active core fields + commented advanced ones); the file documents the full config surface and is a no-op when copied as-is (PR #164)
- [~] **[L]** Evaluated auto-generating internals from `///` doc comments — **won't do**: `cargo doc` produces an API reference for public items, but `INTERNALS.md` is a hand-written architecture *narrative* (request lifecycle, middleware stack, design rationale) it can't replace; rucho is a binary so the generated docs aren't published anywhere either. Keeping INTERNALS hand-written. No split for now — it's large but well-sectioned with a ToC; revisit only if it becomes unwieldy

---

## Priority Order

Ranked by payoff for the dual mission:

Every prioritized tier is now shipped: **T1** (Echo & Inspection Fidelity, incl. the TLS-info echo), **T4** (Testing & Quality — integration/property/benchmark coverage + an MSRV CI job), and **T6** (Documentation — the dual-mission story, the dedup pass, and polish), plus the **T3** code-quality hygiene fixes. What remains is optional, scattered, low-priority polish across **T2 / T3 / T5** — metrics `DashMap`, auto-cert, an Alpine image, `SHA256SUMS`, a `/metrics` Swagger annotation, HTTPS TCP tuning, handler-DRY, module reorg — none dominating payoff. Pick any if/when the itch arises.

_Done: `windows-latest` CI (#136) · "Why rucho?" + Kong docs (#137) · `spawn_full_app()` + lib refactor (#138) · multi-arch Docker (#139) · `/status` + `/redirect` (#140) · amd64-only PR CI (#141) · forced-encoding trio (#142) · metrics cardinality cap (#143) · `/cache` (#144) · cookie fidelity (#145) · ROADMAP reconcile (#146) · request-id middleware (#147) · SIGTERM shutdown (#148) · `log_format=json` (#149) · read-only-FS PID compat (#150) · `http_version` echo (#151) · `X-Response-Time` header (#152) · MSRV alignment (#153) · TLS-info echo (#155) · integration-test gaps (#156) · property tests (#157) · benchmark gaps (#158) · MSRV CI job (#159) · project-structure dedup (#160) · config-table dedup (#161) · API_REFERENCE one-pager (#162) · README feature grouping (#163) · config_samples style (#164) · CHANGELOG perf + compare-links (#165) · SECURITY.md (#166) · T3 hygiene (#167) · connection-close knob (#170)._

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
