# Contributing to Rucho

Thank you for your interest in contributing to Rucho! This document provides guidelines and information for contributors.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/rucho.git
   cd rucho
   ```
3. **Add the upstream remote**:
   ```bash
   git remote add upstream https://github.com/rumpus/rucho.git
   ```

## Development Setup

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Cargo (comes with Rust)

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run clippy lints
cargo clippy

# Format code
cargo fmt

# Run benchmarks
cargo bench
```

### Running Locally

```bash
# Start the server
cargo run -- start

# Check status
cargo run -- status

# Stop the server
cargo run -- stop
```

## Making Changes

### Branch Naming

Use descriptive branch names:
- `feature/add-cookie-endpoint`
- `fix/delay-endpoint-timeout`
- `docs/update-readme`
- `refactor/cli-module`

### Commit Messages

Write clear, concise commit messages:
- Use present tense ("Add feature" not "Added feature")
- Use imperative mood ("Fix bug" not "Fixes bug")
- Keep the first line under 72 characters
- Reference issues when applicable

Examples:
```
Add cookie inspection endpoint

Implement /cookies endpoint for inspecting and setting HTTP cookies.
Includes tests and documentation updates.

Closes #42
```

### Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and address warnings
- Follow Rust naming conventions
- Add documentation comments for public APIs
- Keep functions focused and reasonably sized

### Testing

- Add tests for new functionality
- Ensure all existing tests pass: `cargo test`
- Config tests use an injectable env reader — no need for `--test-threads=1`

## Pull Request Process

1. **Update your fork** with the latest upstream changes:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature
   ```

3. **Make your changes** and commit them

4. **Push to your fork**:
   ```bash
   git push origin feature/your-feature
   ```

5. **Open a Pull Request** on GitHub

### PR Requirements

- Clear description of changes
- Tests pass (`cargo test`)
- No clippy warnings (`cargo clippy`)
- Code is formatted (`cargo fmt`)
- Documentation updated if needed

## Project Structure

```
benches/                         # Criterion benchmarks (cargo bench)
├── response_benchmarks.rs       # Response building microbenchmarks
└── endpoint_benchmarks.rs       # Full request cycle benchmarks
src/
├── main.rs              # Application entrypoint
├── lib.rs               # Library exports
├── cli/                 # CLI argument parsing and commands
│   ├── mod.rs
│   └── commands.rs      # start, stop, status, version handlers
├── routes/              # HTTP route handlers
│   ├── mod.rs
│   ├── cookies.rs       # /cookies endpoints
│   ├── core_routes.rs   # Core echo endpoints
│   ├── delay.rs         # /delay/:n endpoint
│   ├── healthz.rs       # /healthz endpoint
│   ├── metrics.rs       # /metrics endpoint handler
│   └── redirect.rs      # /redirect/:n endpoint
├── server/              # Server setup and orchestration
│   ├── mod.rs
│   ├── chaos_layer.rs   # Chaos engineering middleware
│   ├── http.rs          # HTTP/HTTPS listener setup
│   ├── metrics_layer.rs # Metrics collection middleware
│   ├── tcp.rs           # TCP echo listener
│   ├── timing_layer.rs  # Request timing middleware
│   ├── udp.rs           # UDP echo listener
│   └── shutdown.rs      # Graceful shutdown handling
├── tcp_udp_handlers.rs  # TCP/UDP echo protocol handlers
└── utils/               # Utility modules
    ├── mod.rs
    ├── config.rs        # Configuration loading
    ├── constants.rs     # Centralized constants
    ├── error_response.rs
    ├── json_response.rs
    ├── metrics.rs       # Metrics data structures
    ├── pid.rs           # PID file management
    ├── server_config.rs # Listener and TLS configuration
    └── timing.rs        # Timing utilities
```

## Releasing

Releases are automated via GitHub Actions. When a version tag is pushed, the [release workflow](.github/workflows/release.yml) runs fmt/clippy/test, builds a release binary, creates a GitHub release, and pushes a Docker image to Docker Hub.

To cut a new release:

1. Create a release branch (`release/v<version>`)
2. Bump `version` in `Cargo.toml`
3. Move the `## [Unreleased]` section in `CHANGELOG.md` to `## [<version>] - <date>` and add a new empty `## [Unreleased]` above it
4. Merge the PR to main
5. Tag and push:
   ```bash
   git checkout main && git pull
   git tag v<version>
   git push origin v<version>
   ```

The tag push triggers the workflow, which creates the GitHub release with the binary and pushes `rumpus/rucho:<version>` and `rumpus/rucho:latest` to Docker Hub.

## Reporting Issues

When reporting issues, please include:
- Rust version (`rustc --version`)
- Operating system
- Steps to reproduce
- Expected vs actual behavior
- Relevant logs or error messages

## Feature Requests

Feature requests are welcome! Please:
- Check existing issues first
- Describe the use case
- Explain why it would benefit the project

## Questions?

Feel free to open an issue for questions or reach out to the maintainers.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
