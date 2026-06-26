---
description: Cut a rucho release — version bump, CHANGELOG, PR, then tag-push that triggers the GitHub release + Docker publish. Manual-only; never auto-invoked.
argument-hint: <version>
disable-model-invocation: true
---

Cut release **v$ARGUMENTS** of rucho. This ends in an irreversible tag-push that triggers the GitHub release + Docker publish, so it is manual-only.

**Hard guardrails (do not violate):** do NOT `git merge` or `git push` to `main`. Open a PR; the maintainer merges it on GitHub. Push the tag only *after* the release PR is merged (`git tag` is `ask`-gated in `.claude/settings.json`).

1. Branch `release/v$ARGUMENTS`.
2. Bump `version` in `Cargo.toml`.
3. Cut `CHANGELOG.md`: rename `## [Unreleased]` → `## [$ARGUMENTS] - <today>`, add a fresh empty `## [Unreleased]` above, and add the compare-link reference at the bottom.
4. Run the CI-exact gate — `/precommit`.
5. Commit `chore: Prepare v$ARGUMENTS release`, push the branch, open the PR. **Stop here and wait for the maintainer to merge.**
6. After merge: `git checkout main && git pull`, then `git tag v$ARGUMENTS && git push origin v$ARGUMENTS`.
7. The tag push runs `.github/workflows/release.yml`: fmt/clippy/test safety gate → release binary + `.deb` package → GitHub release (CHANGELOG body + binary + `.deb`) → multi-arch Docker image (`rumpus/rucho:$ARGUMENTS` + `:latest`).

**Prerequisites:** repo secrets `DOCKERHUB_USERNAME` + `DOCKERHUB_TOKEN` must be configured.
