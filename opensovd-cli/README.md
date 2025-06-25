# opensovd-cli

CLI tools package for OpenSOVD providing command-line utilities for interacting with SOVD services.

## Overview

This crate provides command-line tools for the OpenSOVD (ISO 17978) implementation, including:

- Gateway daemon for SOVD services
- CLI client for interacting with SOVD servers

## Binaries

| Binary | Description |
|--------|-------------|
| `osovd-gateway` | OpenSOVD daemon/gateway server for handling SOVD service requests |
| `osovd-cli` | OpenSOVD CLI client for interacting with SOVD services and topology management |

## Cargo Features

| Feature | Default | Description |
|---------|---------|-------------|
| `tracing` | Yes | Enables tracing support with journald integration and formatted output |

## Usage

Build all binaries:

```bash
cargo build --package opensovd-cli
```

Run the gateway daemon:

```bash
cargo run --bin osovd-gateway
```

Run the CLI client:

```bash
cargo run --bin osovd-cli
```

To disable tracing:

```toml
[dependencies]
opensovd-cli = { version = "0.0.1", default-features = false }
```
