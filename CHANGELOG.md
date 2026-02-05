# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

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
