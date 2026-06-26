---
description: Add a new HTTP endpoint to rucho end-to-end — handler, OpenAPI annotation, route registration, integration test, and the full doc sweep. Pass the route path as an argument.
argument-hint: <route-path>
---

Add the endpoint `$ARGUMENTS` to rucho. Follow the **"Adding a New Endpoint"** checklist in `CLAUDE.md` as the source of truth — don't skip a step:

1. Handler fn (in an existing route module, or a new `src/routes/<name>.rs`) with a `#[utoipa::path(...)]` annotation.
2. New module only: add `pub mod <name>;` to `src/routes/mod.rs` and give it a `pub fn router() -> Router`.
3. Register the router in `build_app()` in **`src/app.rs`** — `.merge(rucho::routes::<name>::router())`.
4. Add the handler path to the `ApiDoc` `#[openapi(paths(...))]` in **`src/openapi.rs`**.
5. Integration test in `tests/integration.rs` (use the `spawn_app()` helper).
6. Doc sweep: README endpoint table + project tree · `CHANGELOG [Unreleased]` · `ROADMAP` tick + priority rotation · `docs/API_REFERENCE.md` · `docs/INTERNALS.md` endpoint table · `docs/USAGE_EXAMPLES.md`.

Watch the two classic traps (CLAUDE.md → **Common Mistakes**): forgetting the `build_app()` merge → the handler 404s at runtime even though unit tests pass; forgetting the `ApiDoc` path → it's missing from `/swagger-ui`.

Work on a `feature/` branch, then run `/precommit` before committing.
