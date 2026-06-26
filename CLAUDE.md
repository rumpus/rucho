# Rucho — Claude Code Instructions

## Project Overview
Rucho is a Rust-based HTTP echo server and request inspector, similar to httpbin.
It echoes back request details (headers, method, body, IP) as JSON.
Built on **Axum** + Tokio (see `Cargo.toml` for pinned versions), with OpenAPI docs via utoipa, optional TLS, TCP/UDP echo, chaos engineering mode, and metrics.

## Commands
```bash
cargo fmt                          # Format code
cargo clippy -- -D warnings        # Lint (must pass with zero warnings)
cargo test                         # Unit + integration tests
cargo bench                        # Criterion benchmarks (response_benchmarks, endpoint_benchmarks)
cargo run -- start                 # Run server locally (reads ./rucho.conf if present)
```

## Project Map
The full source tree is in **[README.md → Project Structure](README.md#project-structure)** and the architecture deep-dive in **`docs/INTERNALS.md`** — those are the canonical, maintained references. (A hand-kept tree here would drift on every new module, so it points out instead of duplicating.)

**Key files — non-obvious locations worth knowing before you edit:**
- `src/app.rs` — `build_app()`: merges every route `router()` and assembles the middleware stack. **(Not `main.rs`.)**
- `src/openapi.rs` — `ApiDoc` with `#[openapi(paths(...))]`; Swagger UI served at `/swagger-ui`. **(Not `main.rs`.)**
- `src/main.rs` — entry point + CLI dispatch only; thin, the app is built in `app.rs`.
- `src/routes/core_routes.rs` — the core echo handlers (`/get`, `/post`, `/anything`, `/uuid`, `/ip`, …) + the `/endpoints` list.
- `src/utils/server_config.rs` — Rustls **config loader** + `parse_listen_address()`.
- `src/server/tls.rs` — `TlsInfoAcceptor`, which surfaces negotiated TLS to handlers (distinct from the loader above — don't confuse them).
- `src/tcp_udp_handlers.rs` — shared TCP/UDP echo logic.
- `src/utils/json_response.rs` / `error_response.rs` — the response helpers every handler uses.

## Architecture

**Config system**: Custom `key = value` parser (not TOML). Loaded: defaults → `/etc/rucho/rucho.conf` → `./rucho.conf` → env vars (`RUCHO_*`). Toggle pattern: `<feature>_enabled` field + `RUCHO_<FEATURE>_ENABLED` env var.

**Route registration**: Each route module exposes a `router()` fn returning `Router`. All merged in `build_app()` in `src/app.rs`.

**Response pattern**: Handlers return `Response` via `format_json_response(json!({...}))` or `format_json_response_with_timing(data, ms)`. Errors via `format_error_response(StatusCode, "message")`. No central error type.

**Middleware stack** (innermost → outermost): routes → metrics → chaos → timing → trace → compression → cors → normalize-path → request-id. See `src/app.rs` / `docs/INTERNALS.md` for the authoritative order and layer details.

**OpenAPI**: `ApiDoc` struct in `src/openapi.rs` with `#[openapi(paths(...))]`. Swagger UI at `/swagger-ui`.

---

## Code Rules
- Document all public functions with `///` doc comments
- Use `format_json_response()` / `format_error_response()` — never build raw `Response` in handlers
- No `.unwrap()` in production code (tests are fine)
- New config fields: add to `Config` struct, `Default` impl, the file parser, the `load_env_var!` block in `load_from_paths_with_env()` (`src/utils/config.rs`), and `config_samples/rucho.conf.default`
- Keep `config_samples/rucho.conf.default` in sync — CI doesn't check this, so it's easy to forget

## Patterns — Copy These
| Task | Reference file |
|------|---------------|
| New HTTP handler | `src/routes/core_routes.rs` — see any `*_handler` fn |
| New route module | `src/routes/cookies.rs` — `pub fn router() → Router` + utoipa annotations |
| Integration test | `tests/integration.rs` — `spawn_app()` helper, reqwest assertions |
| Route registration | `src/app.rs` `build_app()` — `.merge(module::router())` |
| OpenAPI registration | `src/openapi.rs` `ApiDoc` struct — add path to `#[openapi(paths(...))]` |
| Config toggle | `src/utils/config.rs` `Config` struct — `metrics_enabled` is the canonical example |

## Common Mistakes
1. **Forget route registration** — new handler works in unit tests but 404s at runtime because `build_app()` doesn't merge it
2. **Forget OpenAPI** — endpoint works but missing from `/swagger-ui` because `ApiDoc` paths list wasn't updated
3. **Break config_samples/** — add a config field but forget to add it to `rucho.conf.default`
4. **Allocations in hot paths** — response formatting and metrics paths are perf-sensitive; use `Cow<str>` where possible (see `normalize_path()` in `src/server/metrics_layer.rs`)
5. **Forget to update INTERNALS.md endpoint table** — `docs/INTERNALS.md` has a numbered table of all endpoints; new endpoints often get added to the router/OpenAPI but not the table

## Adding a New Endpoint
_Shortcut: `/add-endpoint <route>` runs this full checklist._
1. Create handler fn in existing route module or new `src/routes/<name>.rs`
2. Add `#[utoipa::path(...)]` annotation to the handler
3. If new module: add `pub mod <name>;` in `src/routes/mod.rs`, create `pub fn router() -> Router`
4. Register in `build_app()` in `src/app.rs`: `.merge(rucho::routes::<name>::router())`
5. Add handler path to `ApiDoc` `#[openapi(paths(...))]` in `src/openapi.rs`
6. Add integration test in `tests/integration.rs`
7. Docs to update: README endpoint table + project tree, CHANGELOG `[Unreleased]`, ROADMAP tick + Suggested Priority Order rotation, `docs/API_REFERENCE.md`, `docs/INTERNALS.md` endpoint table, `docs/USAGE_EXAMPLES.md`

---

## Scope — ask first
Rucho is a **single-maintainer test target**, not a production component. Do NOT add production-team tooling (Dependabot, `cargo audit`, extra CI jobs, enforcement hooks, monitoring integrations) without asking whether it fits this scope. Default to the smallest change that solves the problem.

## Claude Code config (`.claude/`)
This repo's agent config is **checked in** so it ships with the repo and is reviewable:
- **`.claude/settings.json`** (tracked) — the hard git guardrails (deny merge / push-to-`main` / force-push; ask on tag). Don't loosen these here; per-machine convenience allows go in the gitignored `.claude/settings.local.json`.
- **`.claude/commands/`** (tracked) — slash commands: `/precommit` (the CI-exact gate), `/add-endpoint <route>` (the new-endpoint checklist), `/release <version>` (release cutting — `disable-model-invocation`, so it never auto-fires).
- Gitignored / local-only: `CLAUDE.local.md` (personal overrides), `tasks/` (working notes), `.claude/settings.local.json`.

**Config scope (decided deliberately):** slash commands only — **no hooks, subagents, or MCP**. rucho's workflows are short linear checklists, not multi-role investigations or external integrations, so those would add maintenance surface with no payoff for a solo test target. Add one only when a concrete recurring trigger appears (per **Scope — ask first** above).

## Git Workflow
- **Always create a new branch** before making any changes
  - Branch naming: `feature/<description>`, `fix/<description>`, or `docs/<description>`
- **You drive the full commit → push → PR cycle.** For each logical change: commit on the branch, push it to origin, and open the PR with `gh`. I merge on GitHub and report back ("merged"), then you sync `main` and continue. No need to ask before each commit or branch push within this agreed flow.
- **`main` is off-limits for writes.** YOU MUST NOT `git push` to `main` and YOU MUST NOT `git merge` to `main` — PRs only; I merge on GitHub. Branch pushes (`feature/`, `fix/`, `docs/`, `release/`) are expected and pre-authorized.
- **Never force-push** (`--force` / `-f`) — denied in settings.
- **One PR per logical change** — don't bundle unrelated work.
- These guardrails are split across two files: the **hard denials** live in the tracked **`.claude/settings.json`** (`git merge`, push-to-`main`, and force-push are denied; `git tag` prompts) so they ship with the repo and survive context compaction; the per-machine **allows** (routine `cargo`, branch `git push` + `git commit`, read-only `gh`) live in the gitignored **`.claude/settings.local.json`**. Permission precedence is **deny > ask > allow**, so the denials always win. A deny rule is a hard guarantee; the prose above is the advisory restatement.
- **Use conventional commits:**
  - `feat:` new features
  - `fix:` bug fixes
  - `refactor:` code restructuring
  - `docs:` documentation
  - `test:` adding/updating tests
  - `chore:` maintenance tasks

## Before Every Commit
Run these checks and fix any issues (skip for docs-only changes). These match CI exactly — run them or a locally-green change can still red-fail CI:
```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

## Workflow
- Plan mode for non-trivial tasks (3+ steps or architectural decisions); skip it for one-line diffs (a typo, a single ROADMAP tick). If something goes sideways, stop and re-plan.
- Use subagents for research and parallel exploration to keep main context clean.
- Never mark a task complete without proving it works — run tests, check output.
- Fix bugs autonomously: read logs/errors, find root cause, resolve.

## Task Tracking
The `tasks/` directory is **gitignored** (local working notes, not shipped):
- Plan in `tasks/todo.md` with checkable items; mark complete as you go
- Log sessions in `tasks/sessions.md` (date, session number, prompt summaries)
- After **any** user correction, append a Mistake → Correction → Rule entry to `tasks/lessons.md`, and review that file at session start. If the correction is a durable rule, also update this CLAUDE.md directly with a specific, actionable line.

## Context Recovery
When resuming a session:
1. `claude --continue` (most recent session) or `claude --resume` (pick one); `claude --from-pr <n>` resumes a PR-linked session. These restore the full transcript even after `/clear`.
2. Check `git log --oneline -10` and `git status` for current state
3. Read `tasks/todo.md` for in-progress work, `tasks/sessions.md` for recent history

---

## Release Process
Run **`/release <version>`** — the canonical step-by-step checklist lives in `.claude/commands/release.md` (branch → bump `Cargo.toml` → cut `CHANGELOG` → `/precommit` → PR → maintainer merges → tag-push triggers `.github/workflows/release.yml` for the GitHub release + multi-arch Docker publish). It's `disable-model-invocation`, so it only runs when you invoke it by name. Prereq: repo secrets `DOCKERHUB_USERNAME` + `DOCKERHUB_TOKEN`.
