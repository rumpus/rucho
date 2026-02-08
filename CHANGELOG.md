# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.0] - 2026-02-07

### Fixed
- Fix `license-file` path (`LICENSE.md` → `LICENSE`) and copyright year in Debian package metadata
- Remove redundant systemd service asset from Debian package config (already handled by `systemd-units`)

### Changed
- Add `.deb` package build smoke test to CI pipeline
- Attach `.deb` package to GitHub releases
- Consolidate Dockerfile builder layers and use `COPY --chown` to eliminate standalone `RUN chown`
- Harden docker-compose.yml with restart policy, resource limits, logging caps, and read-only config mount
- Add OCI metadata labels to Docker image
- Add `.env*`, coverage artifacts, and compose overrides to `.dockerignore`
- Add Docker build smoke test to CI pipeline

## [1.0.0] - 2026-02-07

### Added

- **Connection Keep-Alive Tuning**: TCP and HTTP connection tuning for performance and resilience
  - TCP keep-alive (idle time, probe interval, retries) via `socket2`
  - TCP_NODELAY to disable Nagle's algorithm (enabled by default)
  - HTTP keep-alive timeout and HTTP/2 ping configuration
  - Header read timeout to protect against slowloris-style attacks
  - All settings configurable via config files and `RUCHO_*` environment variables
- **Chaos Engineering Mode**: Random failure, delay, and response corruption injection for resilience testing
  - Enable via `chaos_mode` config or `RUCHO_CHAOS_MODE` env var (comma-separated: `failure`, `delay`, `corruption`)
  - Failure injection: return configurable HTTP error codes at a set probability
  - Delay injection: add fixed or random delays to requests
  - Response corruption: `empty`, `truncate`, or `garbage` response body mutation
  - `X-Chaos` inform header on affected responses (configurable)
  - Startup validation ensures all required sub-configs are present for enabled types
- **Response Compression**: Optional gzip/brotli compression based on client `Accept-Encoding`
  - Toggle via `compression_enabled` config or `RUCHO_COMPRESSION_ENABLED` env var
  - Off by default to preserve raw response inspection
- **Request Timing**: All JSON echo responses now include processing time
  - `timing.duration_ms` field in response body (sub-millisecond precision)
  - Middleware captures start time, handlers calculate elapsed duration
  - Only applies to JSON echo endpoints (not `/healthz`, `/delay`, `/status`)
- **Metrics Endpoint** (`/metrics`): Request statistics with all-time and rolling hour window
  - Total request count
  - Per-endpoint hit counts
  - Success/failure tracking
  - Path normalization for dynamic routes (`/status/:code`, `/delay/:n`, `/anything/*path`)
  - Toggle via `metrics_enabled` config or `RUCHO_METRICS_ENABLED` env var
- **GitHub Actions CI**: Automated checks (fmt, clippy, test) and build pipeline

### Changed

- **Pretty JSON Default**: All JSON responses are now pretty-printed by default (removed `?pretty=true` query parameter)
- **Docker Optimization**: Multi-stage build reduces image from 3.4GB to 189MB
  - Now available on Docker Hub: `docker pull rumpus/rucho:latest`
  - Added container HEALTHCHECK
  - Added `.dockerignore` for faster builds

### Fixed

- Docker Compose validation errors (removed obsolete `version` attribute, fixed empty environment block)

## [0.1.0] - 2025-02-04

### Added

- **CLI Commands**: `start`, `stop`, `status`, `version` subcommands for server management
- **TCP Echo Listener**: Optional TCP echo server for testing TCP connections
- **UDP Echo Listener**: Optional UDP echo server for testing UDP packets
- **Security Improvements**:
  - Delay endpoint capped at 300 seconds to prevent DoS
  - TCP/UDP buffer size limits (64KB max)
  - Exponential backoff on UDP errors to prevent hot loops
- **Configuration Validation**: SSL cert/key consistency checks
- **PID File Management**: Proper process tracking with `/var/run/rucho/rucho.pid`
- **Centralized Constants**: All magic numbers moved to `src/utils/constants.rs`
- **Modular Architecture**:
  - `src/cli/` - CLI argument parsing and command handlers
  - `src/server/` - Server setup and orchestration
  - `src/utils/pid.rs` - PID file operations
- **Code Quality**:
  - `serialize_headers()` helper to reduce duplication
  - `load_env_var!` macro for DRY config loading
  - Improved error handling (removed unsafe `.unwrap()` calls)

### Changed

- **Project Structure**: Major reorganization for better maintainability
- **main.rs**: Simplified from ~400 lines to ~100 lines
- **Server Config**: SSL flag parsing is now case-insensitive

### Fixed

- Test isolation issues with environment variables
- Potential panic on JSON serialization failures

## [0.0.1] - Initial Release

### Added

- HTTP echo server with Axum framework
- Support for GET, POST, PUT, PATCH, DELETE, OPTIONS, HEAD methods
- `/anything` wildcard endpoint
- `/status/:code` for HTTP status simulation
- `/delay/:n` for response delay testing
- `/healthz` health check endpoint
- `/endpoints` self-documenting endpoint list
- `/swagger-ui` OpenAPI documentation
- HTTPS support via Rustls
- HTTP/2 support (with TLS)
- Configuration via files (`/etc/rucho/rucho.conf`, `./rucho.conf`)
- Configuration via environment variables (`RUCHO_*`)
- Docker support with non-root user
- Docker Compose support
- Systemd service integration
- Graceful shutdown handling
- Request tracing and logging
- CORS support (permissive)
- Pretty JSON output (`?pretty=true`)
