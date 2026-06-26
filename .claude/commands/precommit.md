---
description: Run rucho's CI-exact pre-commit gate (fmt, clippy, test) and fix anything it surfaces. Use before any commit/push to confirm a change is CI-green.
---

Run rucho's CI-exact pre-commit gate and fix anything it surfaces. The `clippy` and `test` invocations match `.github/workflows/ci.yml` verbatim; `cargo fmt` formats in place (CI verifies the same formatting with `cargo fmt --all -- --check`). A locally-green run means CI will pass:

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

- Run **all three** — don't stop at the first. Fix failures, then re-run until clean.
- **Skip only for docs-only changes** (no `.rs` touched).
- If clippy fails in CI but passed locally, your local toolchain is likely behind CI's stable Rust — prefer restructuring the code over `#[allow]` (see `tasks/lessons.md`).
