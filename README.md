<!-- markdownlint-disable -->
<h2 align="center">
    SOVD Playground
</h2>

<div align="center">

[![CI](https://github.com/lh-sag/sovd-playground/workflows/CI/badge.svg)](https://github.com/lh-sag/sovd-playground/actions/workflows/ci.yaml?query=branch%3Amain++)
[![Coverage](https://lh-sag.github.io/sovd-playground/coverage-badge.svg)](https://lh-sag.github.io/sovd-playground)
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

```bash
# Build the workspace
cargo build --workspace

# Run the gateway server from source (default: http://127.0.0.1:9000/sovd)
cargo run --bin sovd-gateway

# Run on dynamic port (OS assigns available port)
cargo run --bin sovd-gateway -- --url http://127.0.0.1:0/sovd
# INFO gw: Add server listening=127.0.0.1:34271 base=/sovd

# Run on abstract Unix domain socket (Linux only)
cargo run --bin sovd-gateway -- --unix-socket @sovd --url http://127.0.0.1/sovd
# INFO gw: Add server listening=@sovd base=/sovd

# Query the API via HTTP
curl --silent --show-error http://127.0.0.1:9000/sovd/v1/components | jq

# Query the API via abstract unix socket
curl --silent --show-error --unix-socket @sovd http://127.0.0.1/sovd/v1/components | jq

# Regenerate UI
npm run build --prefix sovd-ui

# Set up pre-commit hooks (see CONTRIBUTING.md)
pre-commit install
```

## Testing

```bash
# Run Rust unit tests
cargo test --workspace --all-targets --all-features

# Run Python integration tests
uv sync
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
