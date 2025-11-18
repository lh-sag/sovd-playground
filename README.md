<!-- markdownlint-disable -->
<h2 align="center">
    SOVD Playground
</h2>

<div align="center">

[![CI](https://github.com/lh-sag/sovd-playground/workflows/CI/badge.svg?branch=main)](https://github.com/lh-sag/sovd-playground/actions/workflows/ci.yaml?query=branch%3Amain)
[![Coverage](https://lh-sag.github.io/sovd-playground/coverage-badge.svg)](https://lh-sag.github.io/sovd-playground/html/index.html)
[![slack](https://img.shields.io/badge/chat-slack-blue.svg?logo=slack)](https://app.slack.com/client/T02MS1M89UH/C0958MQNGP2)

</div>
<!-- markdownlint-enable -->

Experimental playground for exploring ISO 17978 SOVD (Service-Oriented Vehicle Diagnostics). RESTful API implementation for modern vehicle diagnostics.

## Quick Start

```bash
# Run the gateway server (default: http://127.0.0.1:9000/sovd)
docker run --rm -p 9000:9000 ghcr.io/lh-sag/sovd-gateway:latest

# Query version info
curl --silent --show-error http://127.0.0.1:9000/sovd/v1/version-info | jq

# Open SOVD Dashboard in your browser
# http://127.0.0.1:9000/ui
```

## Development

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install dependencies
uv sync

# Install pre-commit hooks
uv run pre-commit install
```

### Build

```bash
# Build the workspace
cargo build --workspace

# Build UI (generates assets/)
npm run build --prefix sovd-ui
```

### Run

```bash
# HTTP on port 9000
cargo run --bin sovd-gateway -- --url http://127.0.0.1:9000/sovd

# HTTPS
cargo run --bin sovd-gateway -- \
  --url https://127.0.0.1:9443/sovd \
  --cert ./gen/certs/server-cert.pem \
  --key ./gen/certs/server-key.pem \
  --cacert ./gen/certs/ca-cert.pem

# Unix socket (Linux)
cargo run --bin sovd-gateway -- \
  --unix-socket @sovd \
  --url http://localhost/sovd

# Multiple listeners with single config
cargo run --bin sovd-gateway -- --config sovd-cli/gateway/config.toml
```

See `sovd-gateway --help` for all options.

### CURL Examples

```bash
# Get version information
curl --silent --show-error http://127.0.0.1:9000/sovd/version-info | jq

# Discover contained Entities
curl --silent --show-error http://127.0.0.1:9000/sovd/v1 | jq

# List all components
curl --silent --show-error http://127.0.0.1:9000/sovd/v1/components | jq

# Get component details
curl --silent --show-error http://127.0.0.1:9000/sovd/v1/components/engine | jq

# List component data resources
curl --silent --show-error http://127.0.0.1:9000/sovd/v1/components/engine/data | jq

# Read specific data value
curl --silent --show-error http://127.0.0.1:9000/sovd/v1/components/engine/data/rpm | jq
```

### Simple Server Example

See [`examples/server/simple.rs`](examples/server/simple.rs) for a complete example with a mocked engine component.

## Testing

```bash
# Run Rust unit tests
cargo test --workspace --all-targets --all-features

# Run Python integration tests
uv run pytest
```

See [tests/README.md](tests/README.md) for more information.

## CI/CD

Automated workflows handle building, testing, and releases:

- **CI** - Runs on all pushes/PRs (Linux, macOS, Windows)
- **[Latest](https://github.com/lh-sag/sovd-playground/releases/tag/latest)** - Latest releases.
- **[Nightly](https://github.com/lh-sag/sovd-playground/releases/tag/nightly)** - Daily builds.

See [.github/WORKFLOWS.md](.github/WORKFLOWS.md) for workflow architecture and details.

## Project Structure

| Crate | Purpose |
|-------|---------|
| [`sovd-models`](sovd-models/README.md) | SOVD data structures and schemas |
| [`sovd-server`](sovd-server/README.md) | RESTful API |
| [`sovd-diagnostic`](sovd-diagnostic/README.md) | Entity management and diagnostics |
| [`sovd-cli`](sovd-cli/README.md) | Gateway server executable |

## License

Apache-2.0
